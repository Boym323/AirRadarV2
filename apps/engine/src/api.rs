//! G-03.01–07: veřejná projekce jediného RAM stavu a bounded SSE fanout.
//! Žádný druhý ingest/DB; watch drží pouze poslední sdílený frame.
//! Výhradně opt-in --g03-live-api, stávající G-00 bootstrap se nemění.
use crate::contract::PublicAircraft;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::get,
};
use futures_util::{StreamExt, stream};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    convert::Infallible,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{Semaphore, watch};

const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;
const MAX_CLIENTS: usize = 100;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub kind: &'static str,
    pub epoch: String,
    pub seq: u64,
    pub generated_at: String,
    pub aircraft: Vec<PublicAircraft>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Delta {
    pub kind: &'static str,
    pub epoch: String,
    pub seq: u64,
    pub generated_at: String,
    pub upsert: Vec<PublicAircraft>,
    pub removed: Vec<String>,
}

#[derive(Clone)]
struct Frame {
    snapshot: Arc<Snapshot>,
    delta: Option<Arc<Delta>>,
}

#[derive(Clone)]
pub struct Feed {
    rx: watch::Receiver<Arc<Frame>>,
    clients: Arc<Semaphore>,
}

pub struct Publisher {
    sender: watch::Sender<Arc<Frame>>,
    epoch: String,
}

fn unix_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |v| v.as_secs_f64())
}

/// UTC RFC3339 without a new crate/dependency; Unix epoch -> civil Gregorian.
fn timestamp(now: f64) -> String {
    let secs = if now.is_finite() && now >= 0.0 {
        now.floor() as i64
    } else {
        0
    };
    let days = secs.div_euclid(86400);
    let day_seconds = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

pub fn new() -> (Feed, Publisher) {
    let epoch = format!(
        "g03-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |v| v.as_nanos())
    );
    let initial = Arc::new(Frame {
        snapshot: Arc::new(Snapshot {
            kind: "snapshot",
            epoch: epoch.clone(),
            seq: 0,
            generated_at: timestamp(unix_now()),
            aircraft: Vec::new(),
        }),
        delta: None,
    });
    let (sender, rx) = watch::channel(initial);
    (
        Feed {
            rx,
            clients: Arc::new(Semaphore::new(MAX_CLIENTS)),
        },
        Publisher { sender, epoch },
    )
}

impl Publisher {
    /// Jen ingest task (jediný vlastník RAM) volá maximálně jednou za 1 s.
    /// Všechny klienty obsluhuje jeden vypočtený a serializovaný update.
    pub fn publish(
        &mut self,
        aircraft: Vec<PublicAircraft>,
        now: f64,
    ) -> Result<bool, &'static str> {
        let previous = self.sender.borrow().clone();
        if previous.snapshot.aircraft == aircraft {
            return Ok(false);
        }
        let seq = previous
            .snapshot
            .seq
            .checked_add(1)
            .ok_or("sequence overflow")?;
        let generated_at = timestamp(now);
        let snapshot = Snapshot {
            kind: "snapshot",
            epoch: self.epoch.clone(),
            seq,
            generated_at: generated_at.clone(),
            aircraft,
        };
        if serde_json::to_vec(&snapshot)
            .map_err(|_| "snapshot serialize")?
            .len()
            > MAX_SNAPSHOT_BYTES
        {
            return Err("snapshot exceeds 1 MiB");
        }
        let before: BTreeMap<_, _> = previous
            .snapshot
            .aircraft
            .iter()
            .map(|a| (a.id.as_str(), a))
            .collect();
        let after: BTreeMap<_, _> = snapshot
            .aircraft
            .iter()
            .map(|a| (a.id.as_str(), a))
            .collect();
        let upsert = snapshot
            .aircraft
            .iter()
            .filter(|a| before.get(a.id.as_str()) != Some(a))
            .cloned()
            .collect();
        let removed = previous
            .snapshot
            .aircraft
            .iter()
            .filter(|a| !after.contains_key(a.id.as_str()))
            .map(|a| a.id.clone())
            .collect();
        let delta = Delta {
            kind: "delta",
            epoch: self.epoch.clone(),
            seq,
            generated_at,
            upsert,
            removed,
        };
        // A delta may be large; a subscriber can always resynchronize via snapshot.
        let delta = if serde_json::to_vec(&delta)
            .map_err(|_| "delta serialize")?
            .len()
            <= MAX_SNAPSHOT_BYTES
        {
            Some(Arc::new(delta))
        } else {
            None
        };
        self.sender.send_replace(Arc::new(Frame {
            snapshot: Arc::new(snapshot),
            delta,
        }));
        Ok(true)
    }
}

impl Feed {
    fn latest(&self) -> Arc<Snapshot> {
        self.rx.borrow().snapshot.clone()
    }
}

#[derive(Clone)]
struct ApiState {
    feed: Feed,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    stage: &'static str,
    live_ingest: bool,
    database_connected: bool,
}

fn receiver_is_fresh(snapshot: &Snapshot) -> bool {
    let now = unix_now();
    snapshot
        .aircraft
        .iter()
        .any(|aircraft| now >= aircraft.last_message_at && now - aircraft.last_message_at <= 5.0)
}

async fn health(State(state): State<ApiState>) -> Json<Health> {
    let snapshot = state.feed.latest();
    let active = receiver_is_fresh(&snapshot);
    Json(Health {
        status: if active {
            "live"
        } else {
            "waiting_for_observations"
        },
        stage: "G-03",
        live_ingest: active,
        database_connected: false,
    })
}

async fn aircraft(State(state): State<ApiState>) -> Json<serde_json::Value> {
    // HTTP data and status are computed from the same immutable version.
    let snapshot = state.feed.latest();
    let active = receiver_is_fresh(&snapshot);
    Json(serde_json::json!({
        "status": if active { "live" } else { "waiting_for_observations" },
        "data": &*snapshot,
    }))
}

fn sse_event(event: &impl Serialize, kind: &'static str, epoch: &str, seq: u64) -> Event {
    Event::default()
        .event(kind)
        .id(format!("{epoch}:{seq}"))
        .json_data(event)
        .expect("DTO is already serializable")
}

async fn events(State(state): State<ApiState>) -> Result<Response, StatusCode> {
    let permit = state
        .feed
        .clients
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    // subscribe() before reading borrow_and_update: no snapshot->delta race.
    let mut receiver = state.feed.rx.clone();
    let start = receiver.borrow_and_update().clone();
    let last_seq = start.snapshot.seq;
    let first = sse_event(
        &*start.snapshot,
        "snapshot",
        &start.snapshot.epoch,
        last_seq,
    );
    let next = stream::unfold(
        (receiver, last_seq, permit),
        |(mut rx, old_seq, permit)| async move {
            if rx.changed().await.is_err() {
                return None;
            }
            let frame = rx.borrow_and_update().clone();
            let snapshot = &frame.snapshot;
            let event = if snapshot.seq == old_seq.saturating_add(1) {
                match &frame.delta {
                    Some(delta) => sse_event(&**delta, "delta", &snapshot.epoch, snapshot.seq),
                    None => sse_event(&**snapshot, "snapshot", &snapshot.epoch, snapshot.seq),
                }
            } else {
                // watch coalesces by design; missed seq requires a fresh full snapshot.
                sse_event(&**snapshot, "snapshot", &snapshot.epoch, snapshot.seq)
            };
            Some((Ok::<Event, Infallible>(event), (rx, snapshot.seq, permit)))
        },
    );
    let combined = stream::once(async move { Ok::<Event, Infallible>(first) }).chain(next);
    let mut response = Sse::new(combined)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("heartbeat"),
        )
        .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    response
        .headers_mut()
        .insert("x-accel-buffering", HeaderValue::from_static("no"));
    Ok(response)
}

pub fn router(feed: Feed) -> Router {
    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/aircraft", get(aircraft))
        .route("/api/v1/stream", get(events))
        .with_state(ApiState { feed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn fixture_tracks() -> Vec<PublicAircraft> {
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../contracts/fixtures/sse/sequence.json"
        ))
        .expect("fixture");
        serde_json::from_value(source["snapshot"]["aircraft"].clone()).expect("aircraft")
    }

    #[test]
    fn rfc3339_utc_regression() {
        assert_eq!(timestamp(0.0), "1970-01-01T00:00:00Z");
        assert_eq!(timestamp(1792627200.0), "2026-10-22T00:00:00Z");
    }

    #[test]
    fn snapshot_and_delta_match_g01_fixture_and_omit_private_receiver() {
        let (feed, mut publish) = new();
        let initial = feed.latest();
        assert_eq!(initial.seq, 0);
        assert_eq!(initial.aircraft.len(), 0);
        let tracks = fixture_tracks();
        assert!(publish.publish(tracks.clone(), 1792627200.0).unwrap());
        let snapshot = feed.latest();
        assert_eq!(snapshot.seq, 1);
        let value = serde_json::to_value(&*snapshot).unwrap();
        assert_eq!(value["kind"], "snapshot");
        assert_eq!(value["aircraft"][0]["id"], "icao:abc123");
        let as_text = serde_json::to_string(&*snapshot).unwrap();
        for secret in [
            "receiverId",
            "transport",
            "receivedAtUtc",
            "rawFrame",
            "receiverIp",
        ] {
            assert!(!as_text.contains(secret));
        }
        assert!(!publish.publish(tracks.clone(), 1792627200.0).unwrap());
        assert_eq!(feed.latest().seq, 1);
        let mut changed = tracks;
        changed[0].squawk = Some("7700".to_string());
        assert!(publish.publish(changed, 1792627201.0).unwrap());
        let frame = feed.rx.borrow().clone();
        let delta = frame.delta.as_ref().unwrap();
        assert_eq!(delta.upsert.len(), 1);
        assert_eq!(delta.removed.len(), 0);
        assert_eq!(delta.seq, 2);
        assert!(publish.publish(Vec::new(), 1792627202.0).unwrap());
        let frame = feed.rx.borrow().clone();
        assert_eq!(frame.delta.as_ref().unwrap().removed, ["icao:abc123"]);
    }

    #[tokio::test]
    async fn http_empty_and_full_views_and_sse_headers() {
        let (feed, mut publish) = new();
        let app = router(feed.clone());
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/aircraft")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["data"]["aircraft"].as_array().unwrap().len(), 0);
        assert_eq!(json["status"], "waiting_for_observations");
        let mut recent = fixture_tracks();
        recent[0].last_message_at = unix_now();
        publish.publish(recent, unix_now()).unwrap();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/aircraft")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "live");
        assert_eq!(json["data"]["seq"], 1);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/event-stream")
        );
        assert_eq!(
            response.headers()["cache-control"],
            "no-cache, no-transform"
        );
        assert_eq!(response.headers()["x-accel-buffering"], "no");
    }

    #[tokio::test]
    async fn sse_snapshot_delta_and_resync_after_coalesced_seq() {
        let (feed, mut publisher) = new();
        let app = router(feed);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut stream = response.into_body().into_data_stream();
        let first = tokio::time::timeout(Duration::from_millis(400), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let initial = std::str::from_utf8(&first).unwrap();
        assert!(initial.contains("event: snapshot"));
        assert!(initial.contains(r#""seq":0"#));
        publisher.publish(fixture_tracks(), 1792627200.0).unwrap();
        let second = tokio::time::timeout(Duration::from_millis(400), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let delta = std::str::from_utf8(&second).unwrap();
        assert!(delta.contains("event: delta"));
        assert!(delta.contains(r#""seq":1"#));
        // watch intentionally keeps only the newest version for slow clients.
        publisher.publish(Vec::new(), 1792627201.0).unwrap();
        publisher.publish(fixture_tracks(), 1792627202.0).unwrap();
        let third = tokio::time::timeout(Duration::from_millis(400), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let resync = std::str::from_utf8(&third).unwrap();
        assert!(resync.contains("event: snapshot"));
        assert!(resync.contains(r#""seq":3"#));
    }

    #[test]
    fn g01_schema_enum_and_telemetry_privacy() {
        let (feed, mut publisher) = new();
        let mut tracks = fixture_tracks();
        tracks[0].telemetry.ground_speed_kt = Some(180.0);
        publisher.publish(tracks, 1792627200.0).unwrap();
        let snapshot = feed.latest();
        let value = serde_json::to_value(&*snapshot).unwrap();
        let parsed: crate::contract::StreamEvent = serde_json::from_value(value.clone()).unwrap();
        match parsed {
            crate::contract::StreamEvent::Snapshot { aircraft, .. } => {
                assert_eq!(aircraft[0].telemetry.ground_speed_kt, Some(180.0));
            }
            _ => panic!("not a snapshot"),
        }
        let frame = feed.rx.borrow().clone();
        let parsed: crate::contract::StreamEvent =
            serde_json::from_value(serde_json::to_value(&**frame.delta.as_ref().unwrap()).unwrap())
                .unwrap();
        assert!(matches!(parsed, crate::contract::StreamEvent::Delta { .. }));
    }

    #[test]
    fn deltas_reconstruct_exact_canonical_full_state() {
        let (feed, mut publish) = new();
        let mut replica = crate::contract::Replica::default();
        let initial: crate::contract::StreamEvent =
            serde_json::from_value(serde_json::to_value(&*feed.latest()).unwrap()).unwrap();
        replica.apply(initial).unwrap();

        let mut tracks = fixture_tracks();
        for (step, current) in [
            tracks.clone(),
            {
                tracks[0].squawk = Some("7700".into());
                tracks.clone()
            },
            Vec::new(),
        ]
        .into_iter()
        .enumerate()
        {
            publish
                .publish(current, 1792627200.0 + step as f64)
                .unwrap();
            let frame = feed.rx.borrow().clone();
            let delta = frame.delta.as_ref().unwrap();
            let event: crate::contract::StreamEvent =
                serde_json::from_value(serde_json::to_value(&**delta).unwrap()).unwrap();
            replica.apply(event).unwrap();
            let expected: BTreeMap<_, _> = frame
                .snapshot
                .aircraft
                .iter()
                .map(|a| (a.id.clone(), a.clone()))
                .collect();
            assert_eq!(replica.aircraft, expected);
        }
    }

    #[tokio::test]
    async fn client_cap_100_and_releases_permits() {
        let (feed, _) = new();
        let app = router(feed);
        let mut responses = Vec::new();
        for _ in 0..100 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/stream")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            responses.push(response);
        }
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        responses.pop();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
