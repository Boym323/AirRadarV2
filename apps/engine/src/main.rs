//! AirRadar V2 G-00 stub API. No receiver or flight state exists yet.

// G-01 definuje typy před implementací jejich runtime konzumentů v G-02/G-03.
#[allow(dead_code)]
mod contract;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Opt-in G-02 kontrola bez zapojení V1, bez HTTP, bez DB a bez mapy.
    if std::env::args().any(|arg| arg == "--g02-replay") {
        let state = ingest::supervisor::replay()?;
        println!("G-02 fixture replay: tracks={} updated={} duplicates={} conflicts={}",
            state.tracks.len(), state.counters.updated, state.counters.duplicates, state.counters.conflicts);
        return Ok(());
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
