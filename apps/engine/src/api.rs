//! G-03.01–07: veřejná projekce jediného RAM stavu a bounded SSE fanout.
//! Žádný druhý ingest/DB; watch drží pouze poslední sdílený frame.
//! Výhradně opt-in --g03-live-api, stávající G-00 bootstrap se nemění.
use crate::contract::PublicAircraft;
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{RawQuery, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use futures_util::stream;
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
// Per-client application stream owns one reference to shared payload and
// returns at most one <=16KiB Bytes slice per poll. No per-client event queue.
const MAX_CLIENT_PENDING_BYTES: usize = 256 * 1024;
const SSE_CHUNK_BYTES: usize = 16 * 1024;
const _: () = assert!(SSE_CHUNK_BYTES <= MAX_CLIENT_PENDING_BYTES);
const HEARTBEAT: &[u8] = b": heartbeat\n\n";
const CAPACITY_ERROR: &str = "Snímek překročil bezpečný limit; živý stav je dočasně nedostupný.";

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
    #[cfg_attr(not(test), allow(dead_code))]
    delta: Option<Arc<Delta>>,
    // One prepared wire-format SSE payload per frame, shared by all clients.
    encoded_snapshot: Bytes,
    encoded_delta: Option<Bytes>,
    // Do not serve a previous snapshot when the next state exceeds 1 MiB.
    unavailable: bool,
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

// Valid SSE on the wire, encoded ONCE by the publisher and shared as Bytes.
// No Event::json_data per client: a 1MiB frame must not create 100 copies.
fn encode_sse(json: &[u8], kind: &str, epoch: &str, seq: u64) -> Bytes {
    let header = format!("event: {kind}\nid: {epoch}:{seq}\ndata: ");
    let mut result = Vec::with_capacity(header.len() + json.len() + 2);
    result.extend_from_slice(header.as_bytes());
    result.extend_from_slice(json);
    result.extend_from_slice(b"\n\n");
    Bytes::from(result)
}

pub fn new() -> (Feed, Publisher) {
    let epoch = format!(
        "g03-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |v| v.as_nanos())
    );
    let snapshot = Arc::new(Snapshot {
        kind: "snapshot",
        epoch: epoch.clone(),
        seq: 0,
        generated_at: timestamp(unix_now()),
        aircraft: Vec::new(),
    });
    let encoded = serde_json::to_vec(&*snapshot).expect("initial snapshot");
    let initial = Arc::new(Frame {
        encoded_snapshot: encode_sse(&encoded, "snapshot", &epoch, 0),
        snapshot,
        delta: None,
        encoded_delta: None,
        unavailable: false,
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
        if !previous.unavailable && previous.snapshot.aircraft == aircraft {
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
        let encoded = serde_json::to_vec(&snapshot).map_err(|_| "snapshot serialize")?;
        if encoded.len() > MAX_SNAPSHOT_BYTES || snapshot.aircraft.len() > 5000 {
            // Fail closed: replace the last good public frame with an explicit
            // unavailable state. Never leave an apparently live OLD track map.
            // Send a single transition event to existing clients, not 1Hz spam.
            if previous.unavailable {
                return Ok(false);
            }
            let unavailable = Snapshot {
                kind: "snapshot",
                epoch: self.epoch.clone(),
                seq,
                generated_at,
                aircraft: Vec::new(),
            };
            let error = ApiError {
                code: "CAPACITY_EXCEEDED",
                message: CAPACITY_ERROR,
            };
            let error_json = serde_json::to_vec(&error).map_err(|_| "capacity serialize")?;
            self.sender.send_replace(Arc::new(Frame {
                snapshot: Arc::new(unavailable),
                delta: None,
                encoded_snapshot: encode_sse(&error_json, "unavailable", &self.epoch, seq),
                encoded_delta: None,
                unavailable: true,
            }));
            return Err("snapshot exceeds bounded public capacity; API now unavailable");
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
        // When recovering from overflow, force a full snapshot even if seq
        // increments by one; a previous unavailable event invalidated replicas.
        let (delta, encoded_delta) = if previous.unavailable {
            (None, None)
        } else {
            let json = serde_json::to_vec(&delta).map_err(|_| "delta serialize")?;
            if json.len() <= MAX_SNAPSHOT_BYTES {
                (
                    Some(Arc::new(delta)),
                    Some(encode_sse(&json, "delta", &self.epoch, seq)),
                )
            } else {
                (None, None)
            }
        };
        self.sender.send_replace(Arc::new(Frame {
            encoded_snapshot: encode_sse(&encoded, "snapshot", &self.epoch, seq),
            snapshot: Arc::new(snapshot),
            delta,
            encoded_delta,
            unavailable: false,
        }));
        Ok(true)
    }
}

impl Feed {
    #[cfg(test)]
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

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: &'static str,
}

fn error_response(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    let mut response = (status, Json(ApiError { code, message })).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn reject_parameters(query: RawQuery) -> Option<Response> {
    query.0.map(|_| {
        error_response(
            StatusCode::BAD_REQUEST,
            "CONTRACT_INVALID",
            "Tento endpoint nepodporuje query parametry.",
        )
    })
}

async fn health(State(state): State<ApiState>, query: RawQuery) -> Response {
    if let Some(response) = reject_parameters(query) {
        return response;
    }
    let frame = state.feed.rx.borrow().clone();
    let active = !frame.unavailable && receiver_is_fresh(&frame.snapshot);
    Json(Health {
        status: if frame.unavailable {
            "degraded"
        } else if active {
            "live"
        } else {
            "waiting_for_observations"
        },
        stage: "G-03",
        live_ingest: active,
        database_connected: false,
    })
    .into_response()
}

async fn aircraft(State(state): State<ApiState>, query: RawQuery) -> Response {
    if let Some(response) = reject_parameters(query) {
        return response;
    }
    // HTTP data and status are computed from the same immutable version.
    let frame = state.feed.rx.borrow().clone();
    if frame.unavailable {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "CAPACITY_EXCEEDED",
            CAPACITY_ERROR,
        );
    }
    let active = receiver_is_fresh(&frame.snapshot);
    Json(serde_json::json!({
        "status": if active { "live" } else { "waiting_for_observations" },
        "data": &*frame.snapshot,
    }))
    .into_response()
}

// Emits at most one 16KiB slice per poll. The current Bytes refers to ONE
// globally serialized immutable event, not a separate serialized client copy.
// Client-local pending application queue is exactly zero, i.e. <=256KiB.
// TCP/kernel buffers and total RSS require the separate 1/10/100 load gate.
struct SseCursor {
    receiver: watch::Receiver<Arc<Frame>>,
    payload: Option<Bytes>,
    offset: usize,
    seq: u64,
    closing: bool,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

async fn events(State(state): State<ApiState>, query: RawQuery) -> Response {
    if let Some(response) = reject_parameters(query) {
        return response;
    }
    let permit = match state.feed.clients.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "UNAVAILABLE",
                "Limit SSE spojení byl vyčerpán.",
            );
        }
    };
    // Subscribe before reading current frame, so snapshot -> delta is atomic.
    let mut receiver = state.feed.rx.clone();
    let start = receiver.borrow_and_update().clone();
    if start.unavailable {
        return error_response(
            StatusCode::SERVICE_UNAVAILABLE,
            "CAPACITY_EXCEEDED",
            CAPACITY_ERROR,
        );
    }
    let cursor = SseCursor {
        receiver,
        payload: Some(start.encoded_snapshot.clone()),
        offset: 0,
        seq: start.snapshot.seq,
        closing: false,
        _permit: permit,
    };
    let chunks = stream::unfold(cursor, |mut cursor| async move {
        loop {
            if let Some(payload) = &cursor.payload {
                if cursor.offset < payload.len() {
                    let end = (cursor.offset + SSE_CHUNK_BYTES).min(payload.len());
                    let part = payload.slice(cursor.offset..end);
                    cursor.offset = end;
                    if end == payload.len() {
                        cursor.payload = None;
                    }
                    return Some((Ok::<Bytes, Infallible>(part), cursor));
                }
                cursor.payload = None;
            }
            if cursor.closing {
                return None;
            }
            tokio::select! {
                changed = cursor.receiver.changed() => {
                    if changed.is_err() {
                        return None;
                    }
                    let frame = cursor.receiver.borrow_and_update().clone();
                    let next_seq = frame.snapshot.seq;
                    // A missed watch value or degraded-to-live recovery MUST
                    // resynchronize via full snapshot, not an incomplete delta.
                    cursor.payload = Some(
                        if !frame.unavailable && next_seq == cursor.seq.saturating_add(1) {
                            frame.encoded_delta.as_ref().unwrap_or(&frame.encoded_snapshot).clone()
                        } else {
                            frame.encoded_snapshot.clone()
                        }
                    );
                    cursor.seq = next_seq;
                    cursor.offset = 0;
                    if frame.unavailable {
                        // An explicit error event closes the stream. New
                        // subscribers receive 503 until state is valid again.
                        cursor.closing = true;
                    }
                }
                () = tokio::time::sleep(Duration::from_secs(15)) => {
                    cursor.payload = Some(Bytes::from_static(HEARTBEAT));
                    cursor.offset = 0;
                }
            }
        }
    });
    let mut response = Body::from_stream(chunks).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    response
        .headers_mut()
        .insert("x-accel-buffering", HeaderValue::from_static("no"));
    response
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
    use futures_util::StreamExt;
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
    #[tokio::test]
    async fn invalid_query_and_overload_return_versioned_json_errors() {
        let (feed, _) = new();
        let app = router(feed);
        for uri in [
            "/api/v1/health?unknown=true",
            "/api/v1/aircraft?source=other",
            "/api/v1/stream?replay=all",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            assert_eq!(response.headers()["cache-control"], "no-store");
            let body = to_bytes(response.into_body(), 4096).await.unwrap();
            let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(error["code"], "CONTRACT_INVALID");
            assert!(error["message"].as_str().is_some());
        }

        let mut connections = Vec::new();
        for _ in 0..MAX_CLIENTS {
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
            connections.push(response);
        }
        let full = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(full.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(full.into_body(), 4096).await.unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["code"], "UNAVAILABLE");
        drop(connections);
    }

    #[tokio::test]
    async fn reconnect_ignores_old_event_id_and_releases_client_permit() {
        let (feed, mut publisher) = new();
        let app = router(feed.clone());
        let initial = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut old_stream = initial.into_body().into_data_stream();
        let first = old_stream.next().await.unwrap().unwrap();
        assert!(
            std::str::from_utf8(&first)
                .unwrap()
                .contains("event: snapshot")
        );
        drop(old_stream);
        assert_eq!(feed.clients.available_permits(), MAX_CLIENTS);

        publisher.publish(fixture_tracks(), 1792627200.0).unwrap();
        let connected = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .header("last-event-id", "previous-epoch:0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut stream = connected.into_body().into_data_stream();
        let first = stream.next().await.unwrap().unwrap();
        let text = std::str::from_utf8(&first).unwrap();
        assert!(text.contains("event: snapshot"));
        assert!(text.contains(r#""seq":1"#));
        assert_eq!(feed.clients.available_permits(), MAX_CLIENTS - 1);
        drop(stream);
        assert_eq!(feed.clients.available_permits(), MAX_CLIENTS);
    }

    #[tokio::test]
    async fn stalled_reader_gets_full_resync_without_blocking_publisher() {
        let (feed, mut publisher) = new();
        let app = router(feed.clone());
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
        let _ = stream.next().await.unwrap().unwrap();
        let mut tracks = fixture_tracks();
        for i in 1..=64 {
            tracks[0].squawk = Some(format!("{i:04o}"));
            assert!(
                publisher
                    .publish(tracks.clone(), 1792627200.0 + f64::from(i))
                    .unwrap()
            );
        }
        assert_eq!(feed.latest().seq, 64);
        let next = tokio::time::timeout(Duration::from_secs(1), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let event = std::str::from_utf8(&next).unwrap();
        assert!(event.contains("event: snapshot"));
        assert!(event.contains(r#""seq":64"#));
        drop(stream);
        assert_eq!(feed.clients.available_permits(), MAX_CLIENTS);
    }
    #[tokio::test]
    async fn five_thousand_aircraft_shared_snapshot_is_chunked_under_16k() {
        let (feed, mut publisher) = new();
        let mut base = fixture_tracks().remove(0);
        base.position = None;
        base.last_position_at = None;
        base.callsign = None;
        base.squawk = None;
        base.telemetry = crate::contract::FlightData::default();
        let tracks: Vec<_> = (0..5000)
            .map(|i| {
                let mut track = base.clone();
                track.id = format!("icao:{i:06x}");
                track
            })
            .collect();
        assert!(publisher.publish(tracks, 1792627200.0).unwrap());
        let frame = feed.rx.borrow().clone();
        assert_eq!(frame.snapshot.aircraft.len(), 5000);
        assert!(!frame.unavailable);
        // All clients share this payload; each stream yields <=16KiB slices.
        assert!(frame.encoded_snapshot.len() > MAX_CLIENT_PENDING_BYTES);
        assert!(frame.encoded_snapshot.len() < MAX_SNAPSHOT_BYTES + 100);
        let response = router(feed)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let mut stream = response.into_body().into_data_stream();
        let mut combined = Vec::new();
        loop {
            let chunk = tokio::time::timeout(Duration::from_secs(2), stream.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(chunk.len() <= SSE_CHUNK_BYTES);
            combined.extend_from_slice(&chunk);
            if combined.ends_with(b"\n\n") {
                break;
            }
            assert!(combined.len() < MAX_SNAPSHOT_BYTES + 200);
        }
        let message = std::str::from_utf8(&combined).unwrap();
        assert!(message.starts_with("event: snapshot\nid: "));
        assert!(message.ends_with("\n\n"));
        let json = message
            .lines()
            .find_map(|line| line.strip_prefix("data: "))
            .unwrap();
        let snapshot: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(snapshot["aircraft"].as_array().unwrap().len(), 5000);
    }

    #[tokio::test]
    async fn oversize_snapshot_marks_api_degraded_not_stale_and_recovers() {
        let (feed, mut publisher) = new();
        let mut one = fixture_tracks().remove(0);
        one.last_message_at = unix_now();
        assert!(publisher.publish(vec![one.clone()], unix_now()).unwrap());
        let app = router(feed.clone());

        // Existing subscriber must receive explicit unavailable event and close.
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
        let mut stream = response.into_body().into_data_stream();
        assert!(
            std::str::from_utf8(&stream.next().await.unwrap().unwrap())
                .unwrap()
                .contains("event: snapshot")
        );

        let mut dense = Vec::with_capacity(5000);
        one.callsign = Some("ABCDEFGH".to_owned());
        one.telemetry = crate::contract::FlightData {
            altitude_baro_ft: Some(35000),
            ground_speed_kt: Some(455.5),
            track_degrees: Some(179.2),
            vertical_rate_fpm: Some(-1500),
            on_ground: Some(false),
        };
        for i in 0..5000 {
            let mut track = one.clone();
            track.id = format!("icao:{i:06x}");
            dense.push(track);
        }
        assert!(serde_json::to_vec(&dense).unwrap().len() > MAX_SNAPSHOT_BYTES);
        let error = publisher.publish(dense, unix_now()).unwrap_err();
        assert!(error.contains("capacity"));
        assert!(feed.rx.borrow().unavailable);
        assert!(feed.latest().aircraft.is_empty());
        // Repeating overload must not create 1Hz error/event storms.
        // The overloaded state is intentionally explicit (503), not empty live.
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
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let payload = to_bytes(response.into_body(), 4096).await.unwrap();
        let err: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(err["code"], "CAPACITY_EXCEEDED");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let payload = to_bytes(response.into_body(), 4096).await.unwrap();
        let health: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(health["status"], "degraded");
        assert_eq!(health["live_ingest"], false);
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

        let error_event = stream.next().await.unwrap().unwrap();
        let message = std::str::from_utf8(&error_event).unwrap();
        assert!(message.contains("event: unavailable"));
        assert!(message.contains("CAPACITY_EXCEEDED"));
        assert!(stream.next().await.is_none());

        // A valid observation state becomes a fresh full snapshot, not a
        // misleading delta computed against the discarded previous state.
        assert!(publisher.publish(vec![one], unix_now()).unwrap());
        let recovered = feed.rx.borrow().clone();
        assert!(!recovered.unavailable);
        assert!(recovered.delta.is_none());
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
        let mut reconnect = response.into_body().into_data_stream();
        let first = reconnect.next().await.unwrap().unwrap();
        assert!(
            std::str::from_utf8(&first)
                .unwrap()
                .contains("event: snapshot")
        );
    }
}
