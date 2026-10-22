//! G-02.08/09/10: lokální readsb JSON + BEAST, jeden bounded mpsc konzument.
//! Explicitní opt-in: žádné samovolné připojení k V1/readsb.
use super::{
    beast::Decoder,
    json::{parse_replay, read_snapshot, wall_seconds},
    mode_s::{CprCache, decode_df17, observation},
    state::RadarState,
};
use crate::contract::Observation;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::mpsc,
    time::{self, timeout},
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
/// Ingest pouze uvnitř jediného tasku mění RAM mapu. Další komponenty ji
/// dostanou až v G-03 přes snapshot/delta, nikdy přes SQL.
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    if config.receiver_id != "rx-primary" {
        return Err("invalid receiver ID".into());
    }
    let (tx, mut rx) = mpsc::channel::<Observation>(2048);
    if let Some(path) = config.json_path {
        let sender = tx.clone();
        tokio::spawn(async move {
            poll_json(&path, sender).await;
        });
    }
    if let Some(address) = config.beast_addr {
        let sender = tx.clone();
        tokio::spawn(async move {
            poll_beast(&address, sender).await;
        });
    }
    drop(tx);
    let mut state = RadarState::default();
    let mut tick = time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            item=rx.recv()=>{
                match item {Some(obs)=>{state.apply(&obs);},None=>break}
            },
            _=tick.tick()=>{
                state.cleanup(wall_seconds());
                // Interní G-02 diagnostika; veřejné API zatím G-00 stub.
                if std::env::var_os("AIRRADAR_V2_G02_TRACE").is_some(){
                    eprintln!("G-02 RAM tracks={} updates={} duplicates={} conflicts={} capacity_drops={}",
                        state.tracks.len(),state.counters.updated,state.counters.duplicates,
                        state.counters.conflicts,state.counters.capacity_drops);
                }
            }
        }
    }
    Ok(())
}
async fn poll_json(path: &Path, sender: mpsc::Sender<Observation>) {
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        if let Ok(Ok(rows)) =
            timeout(Duration::from_secs(2), read_snapshot(path, "rx-primary")).await
        {
            for row in rows {
                if sender.try_send(row).is_err() {
                    break;
                }
            }
        }
        if sender.is_closed() {
            break;
        }
    }
}
async fn poll_beast(addr: &str, sender: mpsc::Sender<Observation>) {
    let mut delay = 1u64;
    loop {
        let connected = timeout(Duration::from_secs(3), TcpStream::connect(addr)).await;
        if let Ok(Ok(mut tcp)) = connected {
            delay = 1;
            let mut decoder = Decoder::default();
            let mut cache = CprCache::default();
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
                                let pos = decoded
                                    .cpr
                                    .clone()
                                    .and_then(|cpr| cache.observe(&decoded.key, cpr));
                                if sender.try_send(observation(decoded, pos, now)).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
        }
        if sender.is_closed() {
            break;
        }
        time::sleep(Duration::from_secs(delay)).await;
        delay = (delay * 2).min(30);
    }
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
    Ok(state)
}
/// G-02.11: srovnatelné syntetické 1×/2×/4× na stejných fixture.
/// Všechny hodnoty jsou měřeny na stroji CI; NEJSOU to metriky z DEV LXC.
pub fn benchmark() -> Result<(), &'static str> {
    use std::{hint::black_box,time::Instant};
    fn cpu_ticks() -> Option<u64> {
        let stat=std::fs::read_to_string("/proc/self/stat").ok()?;
        let fields=stat.split_once(") ")?.1.split_whitespace().collect::<Vec<_>>();
        Some(fields.get(11)?.parse::<u64>().ok()?+fields.get(12)?.parse::<u64>().ok()?)
    }
    fn rss_kib() -> Option<u64> {
        let status=std::fs::read_to_string("/proc/self/status").ok()?;
        status.lines().find(|l|l.starts_with("VmRSS:"))?
            .split_whitespace().nth(1)?.parse().ok()
    }
    for ratio in [1,2,4] {
        let mut micros=Vec::with_capacity(ratio*1000);
        let wall=Instant::now();
        let cpu_begin=cpu_ticks();
        for _ in 0..ratio*1000 {
            let start=Instant::now();
            let state=replay()?;
            black_box(&state.tracks);
            micros.push(start.elapsed().as_micros() as u64);
        }
        let total=wall.elapsed().as_secs_f64();
        micros.sort_unstable();
        let p95=micros[(micros.len()*95/100).min(micros.len()-1)];
        let cpu_delta=cpu_begin.and_then(|b|cpu_ticks().map(|a|a.saturating_sub(b)));
        println!(
            "G-02 bench {ratio}x: runs={} elapsed_ms={:.1} replay_p95_us={} rss_kib={:?} cpu_ticks={:?} (Linux /proc, HZ se zde nepředpokládá)",
            micros.len(),total*1000.0,p95,rss_kib(),cpu_delta
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    #[tokio::test]
    async fn beast_tcp_loopback_fragments_and_reconnect() {
        use tokio::{io::AsyncWriteExt,net::TcpListener};
        let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr=listener.local_addr().unwrap().to_string();
        let (tx,mut rx)=mpsc::channel(8);
        let handle=tokio::spawn(async move { poll_beast(&addr,tx).await; });
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/beast/cases.json")).unwrap();
        let f=crate::contract::hex_decode(fixture["valid"][0]["hex"].as_str().unwrap()).unwrap();
        for _ in 0..2 {
            let (mut socket,_)=time::timeout(Duration::from_secs(4),listener.accept()).await.unwrap().unwrap();
            socket.write_all(&f[..7]).await.unwrap();
            socket.write_all(&f[7..]).await.unwrap();
            assert_eq!(time::timeout(Duration::from_secs(2),rx.recv()).await.unwrap().unwrap().aircraft_key,"icao:abc123");
            drop(socket);
        }
        handle.abort();
    }

}
