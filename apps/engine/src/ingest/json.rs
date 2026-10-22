//! G-02.02: atomický readsb JSON snapshot, velikostní/časové limity.
use crate::contract::{FlightData, Observation, aircraft_key, normalize_readsb};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;

pub fn wall_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |t| t.as_secs_f64())
}

/// Jeden soubor se nejprve načte celý do RAM (readsb publikuje rename).
/// Závod čtení/rename je validní snapshot staré *nebo* nové verze.
pub async fn read_snapshot(
    path: &Path,
    receiver_id: &str,
) -> Result<Vec<Observation>, &'static str> {
    let meta = tokio::fs::metadata(path)
        .await
        .map_err(|_| "json metadata")?;
    if !meta.is_file() || meta.len() > MAX_JSON_BYTES {
        return Err("json size");
    }
    let bytes = tokio::fs::read(path).await.map_err(|_| "json read")?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err("json size");
    }
    parse_replay(&bytes, receiver_id, wall_seconds())
}

pub fn parse_replay(
    bytes: &[u8],
    receiver_id: &str,
    clock: f64,
) -> Result<Vec<Observation>, &'static str> {
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err("json size");
    }
    let value = serde_json::from_slice(bytes).map_err(|_| "json parse")?;
    let mut observations = normalize_readsb(&value, receiver_id, clock)?;
    let mut extras = std::collections::BTreeMap::new();
    if let Some(rows) = value.get("aircraft").and_then(serde_json::Value::as_array) {
        for row in rows {
            if let Some(key) = row
                .get("hex")
                .and_then(serde_json::Value::as_str)
                .and_then(aircraft_key)
            {
                extras.insert(key, FlightData::from_readsb(row));
            }
        }
    }
    for observation in &mut observations {
        if let Some(info) = extras.get(&observation.aircraft_key) {
            observation.telemetry = info.clone();
        }
    }
    Ok(observations)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn golden_and_bounded_snapshot() {
        let data = include_bytes!("../../../../contracts/fixtures/readsb/initial.json");
        let observations = parse_replay(data, "rx-primary", 1792627200.0).unwrap();
        assert_eq!(observations.len(), 3);
        assert_eq!(observations[0].aircraft_key, "icao:abc123");
        assert_eq!(observations[1].aircraft_key, "nonicao:abc123");
        assert_eq!(observations[0].telemetry.altitude_baro_ft, Some(10000));
        assert_eq!(observations[0].telemetry.ground_speed_kt, Some(180.0));
        assert_eq!(observations[0].telemetry.track_degrees, Some(270.0));
        assert_eq!(observations[2].telemetry.on_ground, Some(true));
        assert_eq!(observations[2].telemetry.track_degrees, Some(0.0));
        assert!(
            parse_replay(
                &vec![b' '; MAX_JSON_BYTES as usize + 1],
                "rx-primary",
                1792627200.0
            )
            .is_err()
        );
        assert!(parse_replay(b"{", "rx-primary", 1792627200.0).is_err());
    }
    #[tokio::test]
    async fn skutečný_readsb_file_a_chybný_snapshot() {
        let file = std::env::temp_dir().join(format!(
            "airradar-g02-json-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let content = serde_json::json!({
            "now":wall_seconds(), "messages":1,
            "aircraft":[{"hex":"abc123","seen":0.0}]
        });
        tokio::fs::write(&file, content.to_string()).await.unwrap();
        let observations = read_snapshot(&file, "rx-primary").await.unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].aircraft_key, "icao:abc123");
        tokio::fs::write(&file, b"{").await.unwrap();
        assert!(read_snapshot(&file, "rx-primary").await.is_err());
        tokio::fs::remove_file(&file).await.unwrap();
    }
}
