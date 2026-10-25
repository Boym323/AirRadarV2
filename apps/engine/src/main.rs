//! AirRadar V2: G-00 bootstrap by default; G-03 HTTP/SSE opt-in over G-02 RAM.

// G-01 definuje typy před implementací jejich runtime konzumentů v G-02/G-03.
#[allow(dead_code)]
mod contract;

mod api;
#[allow(dead_code)]
mod history;
mod history_store;
#[allow(dead_code)]
mod ingest;

use std::{convert::Infallible, net::SocketAddr, time::Duration};

use axum::{
    Json, Router,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures_util::{StreamExt, stream};
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    stage: &'static str,
    live_ingest: bool,
    database_connected: bool,
}

fn health_info() -> Health {
    Health {
        status: "bootstrap",
        stage: "G-00",
        live_ingest: false,
        database_connected: false,
    }
}

async fn health() -> Json<Health> {
    Json(health_info())
}

async fn aircraft() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "NOT_IMPLEMENTED",
        "aircraft": [],
        "source": "bootstrap",
    }))
}

async fn stream_stub() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let initial = Event::default().event("snapshot").data(
        serde_json::json!({
            "sequence": 0,
            "status": "NOT_IMPLEMENTED",
            "aircraft": []
        })
        .to_string(),
    );

    let events = stream::once(async { Ok::<Event, Infallible>(initial) })
        .chain(stream::pending::<Result<Event, Infallible>>());

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    )
}

fn app() -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/aircraft", get(aircraft))
        .route("/api/v1/stream", get(stream_stub))
}

async fn monitor_database(pool: sqlx::PgPool, feed: api::Feed) {
    // G-04/J09: oddělená read-only kontrola, bez dotazů na každý event.
    let mut tick = tokio::time::interval(Duration::from_secs(15));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        let online = matches!(
            tokio::time::timeout(
                Duration::from_secs(3),
                sqlx::query("SELECT 1").execute(&pool),
            )
            .await,
            Ok(Ok(_))
        );
        feed.set_database_online(online);
    }
}

async fn shutdown_requested() {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

// G-03/J06–J08 a G-04/J09: nečekaný konec ingestu musí ukončit HTTP
// s chybou a umožnit systemd provést Restart=on-failure.
async fn run_supervised_live(
    config: ingest::supervisor::Config,
    listener: tokio::net::TcpListener,
    router: Router,
    publisher: api::Publisher,
    history_pool: Option<sqlx::PgPool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (ingest_stop_tx, ingest_stop_rx) = tokio::sync::oneshot::channel::<()>();
    let (http_stop_tx, http_stop_rx) = tokio::sync::oneshot::channel::<()>();
    let mut ingest_task = tokio::spawn(async move {
        ingest::supervisor::run_with_publisher_and_history_until(
            config,
            Some(publisher),
            history_pool,
            async move {
                let _ = ingest_stop_rx.await;
            },
        )
        .await
        .map_err(|error| error.to_string())
    });
    let mut http_task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = http_stop_rx.await;
            })
            .await
            .map_err(|error| error.to_string())
    });
    tokio::select! {
        actor = &mut ingest_task => {
            http_task.abort();
            let _ = http_task.await;
            eprintln!("G-03/G-04 ingest skončil nečekaně: {actor:?}");
            Err("Živý ingest skončil, HTTP služba vyžaduje restart".into())
        }
        api = &mut http_task => {
            let _ = ingest_stop_tx.send(());
            let drained = tokio::time::timeout(Duration::from_secs(65), &mut ingest_task).await;
            if drained.is_err() { ingest_task.abort(); }
            eprintln!("G-03/G-04 HTTP skončilo nečekaně: {api:?}; drain={drained:?}");
            Err("HTTP služba skončila nečekaně".into())
        }
        _ = shutdown_requested() => {
            let _ = ingest_stop_tx.send(());
            let _ = http_stop_tx.send(());
            let drained = tokio::time::timeout(Duration::from_secs(65), &mut ingest_task).await;
            let http = tokio::time::timeout(Duration::from_secs(10), &mut http_task).await;
            let ok = matches!(drained, Ok(Ok(Ok(())))) && matches!(http, Ok(Ok(Ok(()))));
            if !ok {
                ingest_task.abort();
                http_task.abort();
                eprintln!("G-03/G-04 shutdown failed: ingest={drained:?} HTTP={http:?}");
                return Err("G-03/G-04 shutdown nebyl potvrzen".into());
            }
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Opt-in G-02 kontrola bez zapojení V1, bez HTTP, bez DB a bez mapy.
    if std::env::args().any(|arg| arg == "--g02-benchmark") {
        ingest::supervisor::benchmark()?;
        ingest::supervisor::loopback_soak().await?;
        return Ok(());
    }
    if std::env::args().any(|arg| arg == "--g02-replay") {
        let state = ingest::supervisor::replay()?;
        println!(
            "G-02 fixture replay: tracks={} updated={} duplicates={} conflicts={}",
            state.tracks.len(),
            state.counters.updated,
            state.counters.duplicates,
            state.counters.conflicts
        );
        return Ok(());
    }
    // G-04: pouze explicitní DEV režim a samostatná V2 DB v důvěryhodné LAN.
    // Běžný G-03 DEV proces nemá spojení s PostgreSQL.
    if std::env::args().any(|arg| arg == "--g04-dev-history") {
        let config = ingest::supervisor::Config::from_env()?;
        let pool = history_store::dev_pool_from_env()?;
        // Fail closed before switching the main service: the health flag must
        // not claim an active DB runtime when the protected DEV DB is absent.
        pool.acquire().await?;
        let (feed, publisher) = api::new();
        feed.set_sources_configured(config.json_path.is_some(), config.beast_addr.is_some());
        let bind =
            std::env::var("AIRRADAR_V2_BIND").unwrap_or_else(|_| "127.0.0.1:4102".to_owned());
        let listener = tokio::net::TcpListener::bind(bind).await?;
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let mut ingest_task = tokio::spawn(async move {
            match ingest::supervisor::run_with_publisher_and_history_until(
                config,
                Some(publisher),
                Some(pool),
                async move {
                    let _ = stop_rx.await;
                },
            )
            .await
            {
                Ok(()) => true,
                Err(error) => {
                    eprintln!("G-04 ingest/drain failed: {error}");
                    false
                }
            }
        });
        let mut api_task =
            tokio::spawn(async move { axum::serve(listener, api::router(feed)).await });
        // Zastavit přijímání až po minimálně 1800s důkazním běhu.
        // +2s ochrana přesnosti počátečního diagnostického timestampu;
        // 60s drain + 3s SQL close musí skončit PŘED systemd 1875s.
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1802)) => {
                eprintln!("G-04 canary: completed live window, initiating bounded SQL drain");
                let _ = stop_tx.send(());
            }
            actor = &mut ingest_task => {
                api_task.abort();
                eprintln!("G-04 ingest ended before timed shutdown: {actor:?}");
                return Err("G-04 ingest skončil před plánovaným drainem".into());
            }
            api = &mut api_task => {
                ingest_task.abort();
                eprintln!("G-04 HTTP server ended before timed shutdown: {api:?}");
                return Err("G-04 API skončilo před plánovaným drainem".into());
            }
        }
        let drained = tokio::time::timeout(Duration::from_secs(65), &mut ingest_task).await;
        api_task.abort();
        match drained {
            Ok(Ok(true)) => {
                eprintln!("G-04 canary: successful bounded shutdown");
                return Ok(());
            }
            other => {
                eprintln!("G-04 canary: shutdown incomplete: {other:?}");
                return Err("G-04 drain nebyl potvrzen v časovém limitu".into());
            }
        }
    }
    // G-03: opt-in HTTP/SSE, jediný Rust RAM owner.
    if std::env::args().any(|arg| arg == "--g03-live-api") {
        let config = ingest::supervisor::Config::from_env()?;
        let (feed, publisher) = api::new();
        feed.set_sources_configured(config.json_path.is_some(), config.beast_addr.is_some());
        let bind =
            std::env::var("AIRRADAR_V2_BIND").unwrap_or_else(|_| "127.0.0.1:4102".to_owned());
        let listener = tokio::net::TcpListener::bind(bind).await?;
        println!("AirRadar V2 G-03 API on {}", listener.local_addr()?);
        return run_supervised_live(config, listener, api::router(feed), publisher, None).await;
    }
    // G-04: trvalý DEV writer; DB health se ověřuje i po startupu.
    if std::env::args().any(|arg| arg == "--g04-live-history") {
        let config = ingest::supervisor::Config::from_env()?;
        let pool = history_store::dev_pool_from_env()?;
        pool.acquire().await?;
        let (feed, publisher) = api::new();
        feed.set_sources_configured(config.json_path.is_some(), config.beast_addr.is_some());
        feed.set_database_online(true);
        let monitor = tokio::spawn(monitor_database(pool.clone(), feed.clone()));
        let bind =
            std::env::var("AIRRADAR_V2_BIND").unwrap_or_else(|_| "127.0.0.1:4102".to_owned());
        let listener = tokio::net::TcpListener::bind(bind).await?;
        println!("AirRadar V2 G-04 API on {}", listener.local_addr()?);
        let outcome = run_supervised_live(
            config,
            listener,
            api::router_with_runtime(feed, "G-04", true),
            publisher,
            Some(pool),
        )
        .await;
        monitor.abort();
        return outcome;
    }
    // Jen ruční diagnostický režim; běžný systemd vždy zůstává G-00 stub.
    if std::env::args().any(|arg| arg == "--g02-live") {
        let config = ingest::supervisor::Config::from_env()?;
        return ingest::supervisor::run(config).await;
    }
    let bind = std::env::var("AIRRADAR_V2_BIND").unwrap_or_else(|_| "127.0.0.1:4101".to_owned());
    let address: SocketAddr = bind.parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "AirRadar V2 G-00 stub listening on {}",
        listener.local_addr()?
    );
    axum::serve(listener, app())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn unexpected_ingest_end_closes_http() {
        let config = ingest::supervisor::Config {
            receiver_id: "invalid-test-receiver".to_owned(),
            beast_addr: None,
            json_path: None,
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (feed, publisher) = api::new();
        feed.set_sources_configured(config.json_path.is_some(), config.beast_addr.is_some());
        let result = tokio::time::timeout(
            Duration::from_secs(3),
            run_supervised_live(config, listener, api::router(feed), publisher, None),
        )
        .await
        .expect("supervisor must detect an ingest exit");
        assert!(result.is_err());
        assert!(tokio::net::TcpStream::connect(address).await.is_err());
    }

    #[tokio::test]
    async fn health_explicitly_reports_no_live_ingest() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["live_ingest"], false);
        assert_eq!(json["status"], "bootstrap");
    }

    #[tokio::test]
    async fn empty_aircraft_must_not_be_confused_with_live_data() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/aircraft")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(json["status"], "NOT_IMPLEMENTED");
        assert_eq!(json["aircraft"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn stream_uses_sse_content_type() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/event-stream")
        );
    }
}
