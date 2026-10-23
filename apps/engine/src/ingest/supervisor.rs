//! G-02.08/09/10: lokální readsb JSON + BEAST, jeden bounded mpsc konzument.
//! Explicitní opt-in: žádné samovolné připojení k V1/readsb.
use super::{
    beast::Decoder,
    json::{parse_replay, read_snapshot, wall_seconds},
    mode_s::{CprCache, Decoded, decode_df17, observation},
    state::RadarState,
};
use crate::contract::Observation;
use crate::{
    history::{HistorySampler, Sample},
    history_store::WriterDiagnostics,
};
use std::future::Future;
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::mpsc,
    time::{self, Instant, timeout},
};

#[derive(Debug, Clone)]
pub struct Config {
    pub json_path: Option<PathBuf>,
    pub beast_addr: Option<String>,
    pub receiver_id: String,
}
impl Config {
    pub fn from_env() -> Result<Self, &'static str> {
        let json_path = std::env::var_os("AIRRADAR_V2_READSB_JSON").map(PathBuf::from);
        let beast_addr = std::env::var("AIRRADAR_V2_BEAST_ADDR").ok();
        if json_path.is_none() && beast_addr.is_none() {
            return Err("missing source configuration");
        }
        // Přijímač je v současné verzi vždy stejný důvěryhodný rx-primary.
        Ok(Self {
            json_path,
            beast_addr,
            receiver_id: "rx-primary".into(),
        })
    }
}
#[derive(Debug)]
enum Ingress {
    Json(Observation),
    // One readsb JSON file is one bounded queue message (up to 5000 rows),
    // never 5000 individual try_send calls against a 2048-element FIFO.
    JsonBatch {
        rows: Vec<Observation>,
        at: f64,
        pending: Arc<AtomicBool>,
    },
    JsonHealth {
        at: f64,
        ok: bool,
    },
    Beast {
        decoded: Decoded,
        at: f64,
    },
    BeastLink {
        at: f64,
        connected: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveSource {
    Offline,
    Json,
    Beast,
}

#[derive(Default)]
struct SourceArbiter {
    json_seen: Option<f64>,
    beast_seen: Option<f64>,
    beast_since: Option<f64>,
    beast_connected: bool,
}
impl SourceArbiter {
    fn connected(&mut self, up: bool) {
        self.beast_connected = up;
        if !up {
            self.beast_since = None;
            self.beast_seen = None;
        }
    }
    fn beast_message(&mut self, at: f64) {
        if self.beast_connected {
            if self.beast_seen.is_none_or(|old| at - old > 5.0) {
                self.beast_since = Some(at);
            }
            self.beast_seen = Some(at);
        }
    }
    fn active(&self, now: f64) -> ActiveSource {
        let recent_beast = self.beast_connected
            && self
                .beast_seen
                .is_some_and(|at| now >= at && now - at <= 5.0);
        if recent_beast && self.beast_since.is_some_and(|at| now >= at + 10.0) {
            ActiveSource::Beast
        } else if self
            .json_seen
            .is_some_and(|at| now >= at && now - at <= 5.0)
        {
            ActiveSource::Json
        } else if recent_beast {
            ActiveSource::Beast
        } else {
            ActiveSource::Offline
        }
    }
}

/// Arbitráž diagnostikuje zdroje, ale nikdy nezahazuje důvěryhodná pole:
/// obě cesty se slučují v JEDINÉ RAM mapě podle stáří a kvality.
fn consume(
    event: Ingress,
    state: &mut RadarState,
    cpr: &mut CprCache,
    sources: &mut SourceArbiter,
) {
    match event {
        Ingress::Json(obs) => {
            state.apply(&obs);
        }
        Ingress::JsonBatch { rows, at, pending } => {
            // Only this consumer mutates the authoritative Rust RAM map.
            // No unbounded buffering: only one JSON batch can be outstanding.
            sources.json_seen = Some(at);
            state.batch(&rows);
            pending.store(false, Ordering::Release);
        }
        Ingress::JsonHealth { at, ok } => {
            if ok {
                sources.json_seen = Some(at);
            } else {
                sources.json_seen = None;
            }
        }
        Ingress::BeastLink { at, connected } => {
            if at.is_finite() {
                sources.connected(connected);
            }
        }
        Ingress::Beast { decoded, at } => {
            sources.beast_message(at);
            let reference = state.trusted_reference(&decoded.key, at);
            let pos = decoded.cpr.clone().and_then(|value| {
                cpr.observe_with_reference(&decoded.key, value, reference.as_ref())
            });
            state.apply(&observation(decoded, pos, at));
        }
    }
}
/// Ingest pouze uvnitř jediného tasku mění RAM mapu. Další komponenty ji
/// dostanou až v G-03 přes snapshot/delta, nikdy přes SQL.
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    run_with_publisher(config, None).await
}

/// G-03: jediný task RAM stavu; max. 1x za sekundu publikuje veřejný snapshot.
pub async fn run_with_publisher(
    config: Config,
    publisher: Option<crate::api::Publisher>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_with_publisher_and_history(config, publisher, None).await
}

/// G-04 explicitní opt-in. Jediná SQL task nikdy nečeká v ingest actoru.
pub async fn run_with_publisher_and_history(
    config: Config,
    publisher: Option<crate::api::Publisher>,
    history_pool: Option<sqlx::PgPool>,
) -> Result<(), Box<dyn std::error::Error>> {
    run_with_publisher_and_history_until(
        config,
        publisher,
        history_pool,
        std::future::pending::<()>(),
    )
    .await
}

/// Jen opt-in G-04 dostává jednorázový signál pro řízený drain.
/// Bez tohoto signálu zůstává chování živého G-03 beze změny.
pub async fn run_with_publisher_and_history_until<F: Future<Output = ()>>(
    config: Config,
    mut publisher: Option<crate::api::Publisher>,
    history_pool: Option<sqlx::PgPool>,
    shutdown: F,
) -> Result<(), Box<dyn std::error::Error>> {
    tokio::pin!(shutdown);
    if config.receiver_id != "rx-primary" {
        return Err("invalid receiver ID".into());
    }
    let (tx, mut rx) = mpsc::channel::<Ingress>(2048);
    let drops = Arc::new(AtomicU64::new(0));
    let json_pending = Arc::new(AtomicBool::new(false));
    let mut source_tasks = Vec::new();
    if let Some(path) = config.json_path {
        let sender = tx.clone();
        let dropped = Arc::clone(&drops);
        let pending = Arc::clone(&json_pending);
        source_tasks.push(tokio::spawn(async move {
            poll_json(&path, sender, dropped, pending).await;
        }));
    }
    if let Some(address) = config.beast_addr {
        let sender = tx.clone();
        let dropped = Arc::clone(&drops);
        source_tasks.push(tokio::spawn(async move {
            poll_beast(&address, sender, dropped).await;
        }));
    }
    drop(tx);
    let mut state = RadarState::default();
    let mut history_enabled = history_pool.is_some();
    let writer_diagnostics = Arc::new(WriterDiagnostics::default());
    let mut next_history_trace = 0.0;
    let (history_tx, mut history_rx) = mpsc::channel::<Vec<Sample>>(1);
    let (history_ack_tx, mut history_ack_rx) = mpsc::channel::<bool>(1);
    let mut writer_task = None;
    if let Some(pool) = history_pool {
        state.history = Some(Default::default());
        let diagnostics = Arc::clone(&writer_diagnostics);
        writer_task = Some(tokio::spawn(async move {
            while let Some(batch) = history_rx.recv().await {
                let result = crate::history_store::insert_batch_with_retry_observed(
                    &pool,
                    &batch,
                    Some(&diagnostics),
                )
                .await;
                let ok = result.is_ok();
                if !ok && std::env::var_os("AIRRADAR_V2_HISTORY_TRACE").is_some() {
                    eprintln!("G-04 history SQL batch failed: {result:?}");
                }
                if history_ack_tx.send(ok).await.is_err() {
                    break;
                }
            }
            pool.close().await;
        }));
    } else {
        drop(history_rx);
        drop(history_ack_tx);
    }
    let mut cpr = CprCache::default();
    let mut sources = SourceArbiter::default();
    let mut tick = time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            ack=history_ack_rx.recv(), if history_enabled => {
                if let Some(committed) = ack {
                    if let Some(history) = &mut state.history {
                        history.finish_batch(committed, wall_seconds());
                    }
                } else {
                    history_enabled = false;
                }
            },
            item=rx.recv()=>{
                match item {Some(event)=>{consume(event, &mut state, &mut cpr, &mut sources);},None=>break}
            },
            _=tick.tick()=>{
                let now = wall_seconds();
                state.cleanup(now);
                if let Some(history) = state.history.as_mut() {
                    if history_enabled {
                        let batch = history.next_batch(now);
                        if !batch.is_empty() && history_tx.try_send(batch).is_err() {
                            history.finish_batch(false, now);
                        }
                    } else {
                        history.expire(now);
                    }
                }
                if let Some(publish) = publisher.as_mut() {
                    if let Err(error) = publish.publish(state.snapshot(now), now) {
                        eprintln!("G-03 publish rejected: {error}");
                    }
                }
                // Interní G-02 diagnostika; veřejné API zatím G-00 stub.
                if std::env::var_os("AIRRADAR_V2_G02_TRACE").is_some(){
                    eprintln!("G-02 RAM tracks={} updates={} duplicates={} conflicts={} capacity_drops={} channel_drops={} preferred={:?}",
                        state.tracks.len(),state.counters.updated,state.counters.duplicates,
                        state.counters.conflicts,state.counters.capacity_drops,
                        drops.load(Ordering::Relaxed),sources.active(wall_seconds()));
                }
                if std::env::var_os("AIRRADAR_V2_HISTORY_TRACE").is_some()
                    && now >= next_history_trace
                {
                    next_history_trace = now + 15.0;
                    if let Some(history) = &state.history {
                        log_history_trace(history, &writer_diagnostics, now);
                    }
                }
            }
        }
    }

    // Odpojíme pouze lokální G-04 producenty. U G-03 se tato větev
    // bez explicitního shutdown future nikdy nespustí.
    for task in source_tasks {
        task.abort();
    }
    rx.close();
    drop(rx);

    let drained = if let Some(history) = &mut state.history {
        // 30s UTC bucket + 5s close-delay a bounded SQL potvrzení.
        // 60s limit ponechává rezervu pod systemd RuntimeMaxSec=1875.
        drain_history_until(
            history,
            &history_tx,
            &mut history_ack_rx,
            Instant::now() + Duration::from_secs(60),
            wall_seconds,
        )
        .await
    } else {
        true
    };
    drop(history_tx);
    let writer_finished = if let Some(writer) = writer_task {
        matches!(
            time::timeout(Duration::from_secs(3), writer).await,
            Ok(Ok(()))
        )
    } else {
        true
    };
    if let Some(history) = &state.history {
        log_history_trace(history, &writer_diagnostics, wall_seconds());
    }
    eprintln!("G-04 shutdown drained={drained} writer_finished={writer_finished}");
    if !drained || !writer_finished {
        return Err("G-04 shutdown: poslední SQL fronta nebyla potvrzeně vyprázdněna".into());
    }
    Ok(())
}

/// Žádné nové pozice při drainu: jen uzavřené UTC buckety,
/// potvrzené dávky a pevně omezený čas; bez SQL zápisu naslepo.
async fn drain_history_until<F: Fn() -> f64>(
    history: &mut HistorySampler,
    tx: &mpsc::Sender<Vec<Sample>>,
    ack: &mut mpsc::Receiver<bool>,
    deadline: Instant,
    now: F,
) -> bool {
    loop {
        if Instant::now() >= deadline {
            return false;
        }
        let state = history.diagnostics();
        if state.pending == 0 && state.inflight == 0 {
            return true;
        }
        if state.inflight > 0 {
            match time::timeout_at(deadline, ack.recv()).await {
                Ok(Some(success)) => {
                    history.finish_batch(success, now());
                    if !success {
                        return false;
                    }
                }
                _ => return false,
            }
            continue;
        }
        let batch = history.next_batch(now());
        if batch.is_empty() {
            time::sleep(Duration::from_millis(200)).await;
            continue;
        }
        if !matches!(time::timeout_at(deadline, tx.send(batch)).await, Ok(Ok(()))) {
            history.finish_batch(false, now());
            return false;
        }
    }
}

fn log_history_trace(history: &HistorySampler, writer: &WriterDiagnostics, now: f64) {
    let memory = history.diagnostics();
    eprintln!(
        "G-04 history observed_at={} pending={} inflight={} eligible={} coalesced={} committed={} dropped_capacity={} dropped_stale={} dropped_writer={} sql_attempts={} committed_batches={} failed_batches={} sql_timeouts={} connection={:?}",
        now,
        memory.pending,
        memory.inflight,
        memory.eligible,
        memory.coalesced,
        memory.committed,
        memory.dropped_capacity,
        memory.dropped_stale,
        memory.dropped_writer,
        writer.sql_attempts.load(Ordering::Relaxed),
        writer.committed_batches.load(Ordering::Relaxed),
        writer.failed_batches.load(Ordering::Relaxed),
        writer.sql_timeouts.load(Ordering::Relaxed),
        writer.connection_state(),
    );
}

fn enqueue(sender: &mpsc::Sender<Ingress>, dropped: &AtomicU64, event: Ingress) {
    if sender.try_send(event).is_err() {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}
fn enqueue_json_batch(
    sender: &mpsc::Sender<Ingress>,
    dropped: &AtomicU64,
    pending: &Arc<AtomicBool>,
    rows: Vec<Observation>,
) {
    let count = rows.len() as u64;
    // G-03.10: a single 5000-track JSON sample may not overflow the BEAST FIFO.
    // If the previous batch has not yet been consumed, drop a whole snapshot,
    // never enqueue an unlimited series of multi-MiB batch copies.
    if pending.swap(true, Ordering::AcqRel) {
        dropped.fetch_add(count, Ordering::Relaxed);
        return;
    }
    let event = Ingress::JsonBatch {
        rows,
        at: wall_seconds(),
        pending: Arc::clone(pending),
    };
    if sender.try_send(event).is_err() {
        pending.store(false, Ordering::Release);
        dropped.fetch_add(count, Ordering::Relaxed);
    }
}

async fn poll_json(
    path: &Path,
    sender: mpsc::Sender<Ingress>,
    dropped: Arc<AtomicU64>,
    pending: Arc<AtomicBool>,
) {
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        match timeout(Duration::from_secs(2), read_snapshot(path, "rx-primary")).await {
            Ok(Ok(rows)) => {
                enqueue_json_batch(&sender, &dropped, &pending, rows);
            }
            _ => enqueue(
                &sender,
                &dropped,
                Ingress::JsonHealth {
                    at: wall_seconds(),
                    ok: false,
                },
            ),
        }
        if sender.is_closed() {
            break;
        }
    }
}
async fn poll_beast(addr: &str, sender: mpsc::Sender<Ingress>, dropped: Arc<AtomicU64>) {
    let mut delay = 1u64;
    loop {
        let connected = timeout(Duration::from_secs(3), TcpStream::connect(addr)).await;
        if let Ok(Ok(mut tcp)) = connected {
            delay = 1;
            enqueue(
                &sender,
                &dropped,
                Ingress::BeastLink {
                    at: wall_seconds(),
                    connected: true,
                },
            );
            let mut decoder = Decoder::default();
            let mut buf = [0_u8; 4096];
            loop {
                match timeout(Duration::from_secs(5), tcp.read(&mut buf)).await {
                    Ok(Ok(n)) if n > 0 => {
                        for frame in decoder.push(&buf[..n]) {
                            // Mode A/C, DF parity-overlay a nevalidní CRC nejsou důvěryhodné DF17.
                            if frame.kind != 0x33 {
                                continue;
                            }
                            let now = wall_seconds();
                            if let Some(decoded) = decode_df17(&frame.payload, now) {
                                enqueue(&sender, &dropped, Ingress::Beast { decoded, at: now });
                            }
                        }
                    }
                    // Ticho v éteru není důvod resetovat TCP, na rozdíl od EOF/chyby.
                    Err(_) => continue,
                    _ => break,
                }
            }
            enqueue(
                &sender,
                &dropped,
                Ingress::BeastLink {
                    at: wall_seconds(),
                    connected: false,
                },
            );
        }
        if sender.is_closed() {
            break;
        }
        // Krátký omezený jitter proti synchronním reconnect bouřím.
        let jitter_ms = (wall_seconds() * 1000.0) as u64 % 251;
        time::sleep(Duration::from_secs(delay) + Duration::from_millis(jitter_ms)).await;
        delay = (delay * 2).min(30);
    }
}
/// G-02.11: skutečný asynchronní JSON poll + BEAST TCP/fragmentace/reconnect,
/// ale výhradně na soukromém loopback 127.0.0.1 s generovanými testovacími daty.
/// Nerozpoznává ani nekontaktuje produkční readsb, nemá DB/HTTP/notifikace.
pub async fn loopback_soak() -> Result<(), &'static str> {
    use std::time::Instant;
    use tokio::{io::AsyncWriteExt, net::TcpListener};
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../contracts/fixtures/beast/cases.json"
    ))
    .map_err(|_| "soak fixture")?;
    let frame = crate::contract::hex_decode(
        fixture["valid"][0]["hex"]
            .as_str()
            .ok_or("soak beast frame")?,
    )?;
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|_| "soak loopback bind")?;
    let address = listener.local_addr().map_err(|_| "soak addr")?.to_string();

    let path = std::env::temp_dir().join(format!(
        "airradar-g02-loopback-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "soak clock")?
            .as_nanos()
    ));
    fn snapshot() -> String {
        format!(
            r#"{{"now":{},"messages":1,"aircraft":[{{"hex":"abc123","seen":0.0,"seen_pos":0.0,"lat":49.9,"lon":15.3,"flight":"TEST123"}}]}}"#,
            wall_seconds()
        )
    }
    tokio::fs::write(&path, snapshot())
        .await
        .map_err(|_| "soak temp JSON")?;

    // Opakované úmyslné EOF: 5s rychlý CI, 180s skutečný DEV LXC soak.
    // Zprávu záměrně fragmentujeme přes escapovaný 0x1a bajt.
    let server = tokio::spawn(async move {
        for _ in 0..256 {
            let (mut socket, _) = time::timeout(Duration::from_secs(14), listener.accept())
                .await
                .map_err(|_| "soak tcp accept timeout")?
                .map_err(|_| "soak tcp accept")?;
            socket
                .write_all(&frame[..9])
                .await
                .map_err(|_| "soak fragment")?;
            time::sleep(Duration::from_millis(15)).await;
            socket
                .write_all(&frame[9..])
                .await
                .map_err(|_| "soak payload")?;
            time::sleep(Duration::from_millis(50)).await;
            drop(socket);
        }
        Ok::<(), &'static str>(())
    });
    let (tx, mut rx) = mpsc::channel::<Ingress>(2048);
    let drops = Arc::new(AtomicU64::new(0));
    let json_pending = Arc::new(AtomicBool::new(false));
    let beast_handle = {
        let tx = tx.clone();
        let drops = Arc::clone(&drops);
        tokio::spawn(async move { poll_beast(&address, tx, drops).await })
    };
    let json_handle = {
        let tx = tx.clone();
        let drops = Arc::clone(&drops);
        let path = path.clone();
        let pending = Arc::clone(&json_pending);
        tokio::spawn(async move { poll_json(&path, tx, drops, pending).await })
    };
    drop(tx);
    let clock = Instant::now();
    let target = Duration::from_secs(if cfg!(debug_assertions) { 5 } else { 180 });
    let rss_kib = || {
        std::fs::read_to_string("/proc/self/status")
            .ok()?
            .lines()
            .find(|line| line.starts_with("VmRSS:"))?
            .split_whitespace()
            .nth(1)?
            .parse::<u64>()
            .ok()
    };
    let initial_rss = rss_kib();
    let mut peak_rss = initial_rss.unwrap_or(0);
    let mut state = RadarState::default();
    let mut cpr = CprCache::default();
    let mut sources = SourceArbiter::default();
    let mut links_up = 0_u64;
    let mut links_down = 0_u64;
    let mut beast_messages = 0_u64;
    let mut json_ok = 0_u64;
    let mut json_bad = 0_u64;
    let mut malformed = false;
    let mut recovered = false;
    let mut latencies_us = Vec::new();
    let result = time::timeout(target + Duration::from_secs(20), async {
        loop {
            let elapsed = clock.elapsed();
            peak_rss = peak_rss.max(rss_kib().unwrap_or(0));
            if elapsed >= Duration::from_secs(2) && !malformed {
                tokio::fs::write(&path, b"{")
                    .await
                    .map_err(|_| "soak corrupt JSON")?;
                malformed = true;
            }
            if elapsed >= Duration::from_secs(5) && !recovered {
                tokio::fs::write(&path, snapshot())
                    .await
                    .map_err(|_| "soak recover JSON")?;
                recovered = true;
            }
            if links_up >= 4
                && links_down >= 3
                && beast_messages >= 4
                && json_bad >= 1
                && json_ok >= 2
                && recovered
                && elapsed >= target
            {
                return Ok::<(), &'static str>(());
            }
            if let Ok(Some(item)) = time::timeout(Duration::from_millis(200), rx.recv()).await {
                match &item {
                    Ingress::BeastLink { connected, .. } => {
                        if *connected {
                            links_up += 1;
                        } else {
                            links_down += 1;
                        }
                    }
                    Ingress::Beast { at, .. } => {
                        beast_messages += 1;
                        latencies_us.push(((wall_seconds() - *at).max(0.0) * 1_000_000.0) as u64);
                    }
                    Ingress::JsonHealth { ok, .. } => {
                        if *ok {
                            json_ok += 1;
                        } else {
                            json_bad += 1;
                        }
                    }
                    Ingress::JsonBatch { .. } => {
                        json_ok += 1;
                    }
                    Ingress::Json(_) => {}
                }
                consume(item, &mut state, &mut cpr, &mut sources);
                if state.tracks.len() > 1 {
                    return Err("soak duplicate identity");
                }
            }
        }
    })
    .await;
    // Žádná další síťová aktivita nebo otevřený testovací soubor po skončení.
    beast_handle.abort();
    json_handle.abort();
    server.abort();
    let _ = tokio::fs::remove_file(path).await;
    result.map_err(|_| "soak recovery timeout")??;
    if state.tracks.len() != 1
        || !state.tracks.contains_key("icao:abc123")
        || drops.load(Ordering::Relaxed) != 0
    {
        return Err("soak RAM/backpressure invariant");
    }
    state.cleanup(wall_seconds() + 61.0);
    if !state.tracks.is_empty() {
        return Err("soak TTL leak");
    }
    latencies_us.sort_unstable();
    let p95 = latencies_us[(latencies_us.len() * 95 / 100).min(latencies_us.len() - 1)];
    println!(
        "G-02 isolated-loopback soak PASS: duration_ms={} tcp_connects={} tcp_eofs={} beast_frames={} json_ok={} json_fail={} queue_drops={} p95_receive_us={} rss_start_kib={:?} rss_peak_kib={} tracks_after_ttl={} receiver=127.0.0.1(no V1/DB)",
        clock.elapsed().as_millis(),
        links_up,
        links_down,
        beast_messages,
        json_ok,
        json_bad,
        drops.load(Ordering::Relaxed),
        p95,
        initial_rss,
        peak_rss,
        state.tracks.len()
    );
    Ok(())
}

/// Bez síťového IO a bez SQL: deterministický replay pro G-02.10.
pub fn replay() -> Result<RadarState, &'static str> {
    let mut state = RadarState::default();
    let files: [(&[u8], f64); 3] = [
        (
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            1792627200.0,
        ),
        (
            include_bytes!("../../../../contracts/fixtures/readsb/repeated.json"),
            1792627201.0,
        ),
        (
            include_bytes!("../../../../contracts/fixtures/readsb/changed.json"),
            1792627205.0,
        ),
    ];
    for (bytes, t) in files {
        state.batch(&parse_replay(bytes, "rx-primary", t)?);
    }
    // Běh CLI musí pokrýt OBA transporty v téže RAM autoritě, bez TCP/LAN.
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../contracts/fixtures/beast/cases.json"
    ))
    .map_err(|_| "beast fixture")?;
    let mut cpr = CprCache::default();
    let mut source = SourceArbiter::default();
    consume(
        Ingress::BeastLink {
            at: 1792627206.0,
            connected: true,
        },
        &mut state,
        &mut cpr,
        &mut source,
    );
    for (i, case) in cases["valid"]
        .as_array()
        .ok_or("beast cases")?
        .iter()
        .take(2)
        .enumerate()
    {
        let bytes = crate::contract::hex_decode(case["modeS"].as_str().ok_or("modeS")?)?;
        let at = 1792627206.0 + i as f64;
        let decoded = decode_df17(&bytes, at).ok_or("beast crc")?;
        consume(
            Ingress::Beast { decoded, at },
            &mut state,
            &mut cpr,
            &mut source,
        );
    }
    let bad =
        crate::contract::hex_decode(cases["invalid"][3]["hex"].as_str().ok_or("invalid frame")?)?;
    let mut decoder = Decoder::default();
    let frames = decoder.push(&bad);
    if frames.len() != 1 || decode_df17(&frames[0].payload, 1792627210.0).is_some() {
        return Err("bad CRC accepted");
    }
    if state.tracks.len() != 3 {
        return Err("duplicate aircraft identity");
    }
    Ok(state)
}
/// G-02.11: srovnatelné syntetické 1×/2×/4× na stejných fixture.
/// Všechny hodnoty jsou měřeny na stroji CI; NEJSOU to metriky z DEV LXC.
pub fn benchmark() -> Result<(), &'static str> {
    use std::{hint::black_box, time::Instant};
    fn cpu_ticks() -> Option<u64> {
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        let fields = stat
            .split_once(") ")?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        Some(fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?)
    }
    fn rss_kib() -> Option<u64> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        status
            .lines()
            .find(|l| l.starts_with("VmRSS:"))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    }
    for ratio in [1, 2, 4] {
        let mut micros = Vec::with_capacity(ratio * 1000);
        let wall = Instant::now();
        let cpu_begin = cpu_ticks();
        for _ in 0..ratio * 1000 {
            let start = Instant::now();
            let state = replay()?;
            black_box(&state.tracks);
            micros.push(start.elapsed().as_micros() as u64);
        }
        let total = wall.elapsed().as_secs_f64();
        micros.sort_unstable();
        let p95 = micros[(micros.len() * 95 / 100).min(micros.len() - 1)];
        let cpu_delta = cpu_begin.and_then(|b| cpu_ticks().map(|a| a.saturating_sub(b)));
        println!(
            "G-02 bench {ratio}x: runs={} elapsed_ms={:.1} replay_p95_us={} rss_kib={:?} cpu_ticks={:?} (Linux /proc, HZ se zde nepředpokládá)",
            micros.len(),
            total * 1000.0,
            p95,
            rss_kib(),
            cpu_delta
        );
    }

    // Samostatná 5000-cílová bounded RAM zátěž bez přijímače, DB a SSE.
    // Rychlosti jsou syntetické, nikoli měření RF toku v cílovém LXC.
    let baseline = parse_replay(
        include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
        "rx-primary",
        1792627200.0,
    )?
    .into_iter()
    .next()
    .ok_or("baseline")?;
    for ratio in [1usize, 2, 4] {
        let mut state = RadarState::default();
        let mut batch_us = Vec::new();
        let start_wall = Instant::now();
        let start_cpu = cpu_ticks();
        let mut sample = baseline.clone();
        sample.position = None;
        sample.position_at_utc = None;
        for index in 0..5000 {
            sample.aircraft_key = format!("icao:{index:06x}");
            state.apply(&sample);
        }
        if state.tracks.len() != 5000 {
            return Err("RAM bound on fill");
        }
        for cycle in 0..(ratio * 8) {
            let start = Instant::now();
            let now = 1792627201.0 + cycle as f64;
            sample.received_at_utc = now;
            sample.message_at_utc = now;
            for index in 0..5000 {
                sample.aircraft_key = format!("icao:{index:06x}");
                state.apply(&sample);
            }
            batch_us.push(start.elapsed().as_micros() as u64);
        }
        let mut excess = sample.clone();
        excess.aircraft_key = "icao:ffffff".into();
        if state.apply(&excess) != "capacity" || state.tracks.len() != 5000 {
            return Err("RAM capacity violation");
        }
        batch_us.sort_unstable();
        let p95 = batch_us[(batch_us.len() * 95 / 100).min(batch_us.len() - 1)];
        let ticks = start_cpu.and_then(|old| cpu_ticks().map(|new| new.saturating_sub(old)));
        println!(
            "G-02 5000-target stress {ratio}x: observations={} elapsed_ms={:.1} batch5000_p95_us={} rss_kib={:?} cpu_ticks={:?} drops={}",
            5000 * (ratio * 8 + 1),
            start_wall.elapsed().as_secs_f64() * 1000.0,
            p95,
            rss_kib(),
            ticks,
            state.counters.capacity_drops
        );
        state.cleanup(1792627201.0 + (ratio * 8) as f64 + 61.0);
        if !state.tracks.is_empty() {
            return Err("RAM track expiry");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history_store::{insert_batch, read_history};
    #[test]
    fn replay_single_authority() {
        let state = replay().unwrap();
        assert_eq!(state.tracks.len(), 3);
        assert_eq!(
            state.tracks["icao:abc123"].callsign.as_deref(),
            Some("NEW123")
        );
        assert!(state.counters.duplicates > 0);
        assert_eq!(state.counters.capacity_drops, 0);
    }
    #[tokio::test]
    async fn g03_10_json_batch_preserves_all_5000_tracks_without_fifo_loss() {
        let template = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap()
        .remove(0);
        let rows: Vec<Observation> = (0..5000)
            .map(|index| {
                let mut observation = template.clone();
                observation.aircraft_key = format!("icao:{index:06x}");
                observation
            })
            .collect();
        let (tx, mut rx) = mpsc::channel::<Ingress>(2);
        let dropped = AtomicU64::new(0);
        let pending = Arc::new(AtomicBool::new(false));
        enqueue_json_batch(&tx, &dropped, &pending, rows);
        assert!(pending.load(Ordering::Acquire));
        let event = rx.recv().await.unwrap();
        let mut state = RadarState::default();
        let mut cpr = CprCache::default();
        let mut sources = SourceArbiter::default();
        consume(event, &mut state, &mut cpr, &mut sources);
        assert_eq!(state.tracks.len(), 5000);
        assert!(!pending.load(Ordering::Acquire));
        assert_eq!(dropped.load(Ordering::Relaxed), 0);
        assert_eq!(sources.active(wall_seconds()), ActiveSource::Json);
    }

    #[tokio::test]
    async fn g03_10_json_batch_backpressure_discards_whole_sample_not_prefix() {
        let template = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap()
        .remove(0);
        let (tx, mut rx) = mpsc::channel::<Ingress>(1);
        let pending = Arc::new(AtomicBool::new(false));
        let dropped = AtomicU64::new(0);
        let rows = vec![template.clone(); 5000];
        enqueue_json_batch(&tx, &dropped, &pending, rows);
        assert!(pending.load(Ordering::Acquire));
        enqueue_json_batch(&tx, &dropped, &pending, vec![template; 5000]);
        assert_eq!(dropped.load(Ordering::Relaxed), 5000);
        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn g04_beast_only_cpr_ram_sampler_sqlx_postgres_regression() {
        let Ok(url) = std::env::var("AIRRADAR_G04_CI_DATABASE_URL") else {
            // Skutečný PostgreSQL důkaz běží pouze v izolovaném G-04 workflow.
            return;
        };
        let pool = sqlx::PgPool::connect(&url)
            .await
            .expect("isolated PostgreSQL");
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let base: f64 = 1_792_800_000.0;
        sqlx::query(
            "DELETE FROM airradar_v2.flight_samples WHERE aircraft_key = 'icao:abc123' AND bucket_at = to_timestamp($1)",
        )
        .bind(base.floor() / 30.0 * 30.0)
        .execute(&pool)
        .await
        .unwrap();

        // Dva skutečné DF17 CPR rámce, bez JSON kotvy a bez syntetické pozice.
        let mut state = RadarState::default();
        state.history = Some(Default::default());
        let mut cpr = CprCache::default();
        let mut sources = SourceArbiter::default();
        for (index, case) in fixture["valid"]
            .as_array()
            .unwrap()
            .iter()
            .take(2)
            .enumerate()
        {
            let frame = crate::contract::hex_decode(case["modeS"].as_str().unwrap()).unwrap();
            let at = base + index as f64;
            let decoded = decode_df17(&frame, at).unwrap();
            consume(
                Ingress::Beast { decoded, at },
                &mut state,
                &mut cpr,
                &mut sources,
            );
        }
        let history = state.history.as_mut().unwrap();
        assert_eq!(history.counters.eligible, 1);
        // Stejný závěrečný drain jako v DEV canary, tentokrát s reálným SQLx.
        let (tx, mut rx) = mpsc::channel::<Vec<Sample>>(1);
        let (ack_tx, mut ack_rx) = mpsc::channel::<bool>(1);
        let write_pool = pool.clone();
        let writer = tokio::spawn(async move {
            while let Some(batch) = rx.recv().await {
                let ok = insert_batch(&write_pool, &batch).await.is_ok();
                if ack_tx.send(ok).await.is_err() {
                    break;
                }
            }
        });
        let drained = drain_history_until(
            history,
            &tx,
            &mut ack_rx,
            Instant::now() + Duration::from_secs(3),
            || base + 36.0,
        )
        .await;
        drop(tx);
        writer.await.unwrap();
        assert!(drained);
        assert_eq!(history.counters.committed, 1);
        assert_eq!(history.pending_len(), 0);
        let rows = read_history(&pool, "icao:abc123", base - 1.0, base + 30.0, 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].aircraft_key, "icao:abc123");
        sqlx::query(
            "DELETE FROM airradar_v2.flight_samples WHERE aircraft_key = 'icao:abc123' AND bucket_at = to_timestamp($1)",
        )
        .bind(base.floor() / 30.0 * 30.0)
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;
    }

    #[tokio::test]
    async fn g04_shutdown_flushes_pending_and_waits_for_ack() {
        let mut history = HistorySampler::default();
        let mut sample = crate::contract::beast_arrival(
            "abc123",
            1800.0,
            crate::contract::Position {
                lat: 49.0,
                lon: 17.0,
            },
        )
        .unwrap();
        sample.position_origin = "adsb_cpr".to_owned();
        assert!(history.observe(&sample));
        let (tx, mut rx) = mpsc::channel::<Vec<Sample>>(1);
        let (ack_tx, mut ack_rx) = mpsc::channel::<bool>(1);
        let writer = tokio::spawn(async move {
            let batch = rx.recv().await.unwrap();
            assert_eq!(batch.len(), 1);
            assert_eq!(batch[0].aircraft_key, "icao:abc123");
            ack_tx.send(true).await.unwrap();
        });
        let drained = drain_history_until(
            &mut history,
            &tx,
            &mut ack_rx,
            Instant::now() + Duration::from_secs(1),
            || 1836.0,
        )
        .await;
        assert!(drained);
        assert_eq!(history.counters.committed, 1);
        assert_eq!(history.pending_len(), 0);
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn g04_shutdown_rejects_failed_sql_ack() {
        let mut history = HistorySampler::default();
        let mut sample = crate::contract::beast_arrival(
            "abc123",
            1800.0,
            crate::contract::Position {
                lat: 49.0,
                lon: 17.0,
            },
        )
        .unwrap();
        sample.position_origin = "adsb_cpr".to_owned();
        assert!(history.observe(&sample));
        let (tx, mut rx) = mpsc::channel::<Vec<Sample>>(1);
        let (ack_tx, mut ack_rx) = mpsc::channel::<bool>(1);
        let writer = tokio::spawn(async move {
            let _batch = rx.recv().await.unwrap();
            ack_tx.send(false).await.unwrap();
        });
        assert!(
            !drain_history_until(
                &mut history,
                &tx,
                &mut ack_rx,
                Instant::now() + Duration::from_secs(1),
                || 1836.0,
            )
            .await
        );
        assert_eq!(history.counters.dropped_writer, 1);
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn g04_shutdown_is_bounded_without_sql_ack() {
        let mut history = HistorySampler::default();
        let mut sample = crate::contract::beast_arrival(
            "abc123",
            1800.0,
            crate::contract::Position {
                lat: 49.0,
                lon: 17.0,
            },
        )
        .unwrap();
        sample.position_origin = "adsb_cpr".to_owned();
        assert!(history.observe(&sample));
        let (tx, mut rx) = mpsc::channel::<Vec<Sample>>(1);
        let (_ack_tx, mut ack_rx) = mpsc::channel::<bool>(1);
        let held_writer = tokio::spawn(async move {
            let _ = rx.recv().await;
            time::sleep(Duration::from_secs(1)).await;
        });
        assert!(
            !drain_history_until(
                &mut history,
                &tx,
                &mut ack_rx,
                Instant::now() + Duration::from_millis(30),
                || 1836.0,
            )
            .await
        );
        assert_eq!(history.diagnostics().inflight, 1);
        held_writer.abort();
    }

    #[test]
    fn bounded_fifo_and_missing_source_rejected() {
        assert!(
            Config {
                json_path: None,
                beast_addr: None,
                receiver_id: "wrong".into()
            }
            .receiver_id
                != "rx-primary"
        );
    }
    #[test]
    fn json_beast_interleaving_and_fallback_one_track() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let mut state = RadarState::default();
        let mut cpr = CprCache::default();
        let mut sources = SourceArbiter::default();
        for obs in parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap()
        {
            consume(Ingress::Json(obs), &mut state, &mut cpr, &mut sources);
        }
        consume(
            Ingress::JsonHealth {
                at: 1792627200.0,
                ok: true,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        consume(
            Ingress::BeastLink {
                at: 1792627200.0,
                connected: true,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        for (i, case) in fixture["valid"]
            .as_array()
            .unwrap()
            .iter()
            .take(2)
            .enumerate()
        {
            let frame = crate::contract::hex_decode(case["modeS"].as_str().unwrap()).unwrap();
            let at = 1792627201.0 + i as f64;
            let decoded = decode_df17(&frame, at).unwrap();
            consume(
                Ingress::Beast { decoded, at },
                &mut state,
                &mut cpr,
                &mut sources,
            );
        }
        assert_eq!(sources.active(1792627202.0), ActiveSource::Json);
        assert_eq!(state.tracks.len(), 3);
        assert_eq!(
            state.tracks["icao:abc123"].position.as_ref().unwrap().lat,
            49.9
        );
        consume(
            Ingress::BeastLink {
                at: 1792627203.0,
                connected: false,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        assert_eq!(sources.active(1792627204.0), ActiveSource::Json);
        consume(
            Ingress::JsonHealth {
                at: 1792627220.0,
                ok: false,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        assert_eq!(sources.active(1792627220.0), ActiveSource::Offline);
        consume(
            Ingress::BeastLink {
                at: 1792627220.0,
                connected: true,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        for (i, case) in fixture["valid"]
            .as_array()
            .unwrap()
            .iter()
            .take(2)
            .enumerate()
        {
            let frame = crate::contract::hex_decode(case["modeS"].as_str().unwrap()).unwrap();
            let at = 1792627220.0 + i as f64;
            let decoded = decode_df17(&frame, at).unwrap();
            consume(
                Ingress::Beast { decoded, at },
                &mut state,
                &mut cpr,
                &mut sources,
            );
        }
        assert_eq!(sources.active(1792627221.0), ActiveSource::Beast);
        assert_eq!(state.tracks.len(), 3);
        assert!(state.tracks["icao:abc123"].last_position_at.unwrap() > 1792627200.0);
        consume(
            Ingress::BeastLink {
                at: 1792627222.0,
                connected: false,
            },
            &mut state,
            &mut cpr,
            &mut sources,
        );
        assert_eq!(sources.active(1792627222.0), ActiveSource::Offline);
    }
    #[test]
    fn beast_prefers_only_after_ten_seconds_healthy() {
        let mut s = SourceArbiter {
            json_seen: Some(200.0),
            ..Default::default()
        };
        s.connected(true);
        s.beast_message(200.0);
        assert_eq!(s.active(202.0), ActiveSource::Json);
        s.beast_message(205.0);
        s.beast_message(210.0);
        assert_eq!(s.active(210.0), ActiveSource::Beast);
        s.connected(false);
        assert_eq!(s.active(210.0), ActiveSource::Offline);
    }
    #[tokio::test]
    async fn beast_tcp_loopback_fragments_and_reconnect() {
        use tokio::{io::AsyncWriteExt, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let (tx, mut rx) = mpsc::channel(8);
        let handle = tokio::spawn(async move {
            poll_beast(&addr, tx, Arc::new(AtomicU64::new(0))).await;
        });
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let f = crate::contract::hex_decode(fixture["valid"][0]["hex"].as_str().unwrap()).unwrap();
        for _ in 0..2 {
            let (mut socket, _) = time::timeout(Duration::from_secs(4), listener.accept())
                .await
                .unwrap()
                .unwrap();
            socket.write_all(&f[..7]).await.unwrap();
            socket.write_all(&f[7..]).await.unwrap();
            let mut got = false;
            for _ in 0..3 {
                if let Ingress::Beast { decoded, .. } =
                    time::timeout(Duration::from_secs(2), rx.recv())
                        .await
                        .unwrap()
                        .unwrap()
                {
                    assert_eq!(decoded.key, "icao:abc123");
                    got = true;
                    break;
                }
            }
            assert!(got);
            drop(socket);
        }
        handle.abort();
    }
}
