//! G-01: důvěryhodné datové kontrakty; runtime ingest vznikne teprve v G-02.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub receiver_id: String,
    pub transport: String,
    pub aircraft_key: String,
    pub received_at_utc: f64,
    pub message_at_utc: f64,
    pub time_basis: String,
    pub position_origin: String,
    pub position_at_utc: Option<f64>,
    pub position: Option<Position>,
    pub callsign: Option<String>,
    pub squawk: Option<String>,
}

fn safe_age(value: Option<&Value>) -> Option<f64> {
    let n = value?.as_f64()?;
    (n.is_finite() && n >= 0.0).then_some(n)
}

pub fn aircraft_key(hex: &str) -> Option<String> {
    let (kind, digits) = match hex.strip_prefix('~') {
        Some(s) => ("nonicao", s),
        None => ("icao", hex),
    };
    (digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
        .then(|| format!("{kind}:{}", digits.to_ascii_lowercase()))
}

pub fn normalize_readsb(
    input: &Value,
    receiver_id: &str,
    received_at: f64,
) -> Result<Vec<Observation>, &'static str> {
    let now = safe_age(input.get("now")).ok_or("now")?;
    if !received_at.is_finite() || (now - received_at).abs() > 10.0 {
        return Err("skew");
    }
    if input.get("messages").and_then(Value::as_u64).is_none() {
        return Err("messages");
    }
    if receiver_id != "rx-primary" {
        return Err("nedůvěryhodná stanice");
    }
    let rows = input
        .get("aircraft")
        .and_then(Value::as_array)
        .ok_or("aircraft")?;
    if rows.len() > 5000 {
        return Err("limit");
    }
    let mut result = Vec::new();
    for row in rows {
        let Some(key) = row
            .get("hex")
            .and_then(Value::as_str)
            .and_then(aircraft_key)
        else {
            continue;
        };
        let Some(seen) = safe_age(row.get("seen")) else {
            continue;
        };
        if seen > 60.0 {
            continue;
        }
        let squawk = match row.get("squawk").and_then(Value::as_str) {
            Some(s) if s.len() == 4 && s.bytes().all(|b| (b'0'..=b'7').contains(&b)) => {
                Some(s.to_owned())
            }
            Some(_) => continue,
            None => None,
        };
        if row.get("lat").is_some() != row.get("lon").is_some() {
            continue;
        }
        let position = match (row.get("lat"), row.get("lon")) {
            (Some(a), Some(b)) => match (a.as_f64(), b.as_f64()) {
                (Some(lat), Some(lon))
                    if lat.is_finite()
                        && lon.is_finite()
                        && (-90.0..=90.0).contains(&lat)
                        && (-180.0..=180.0).contains(&lon) =>
                {
                    Some(Position { lat, lon })
                }
                _ => continue,
            },
            _ => None,
        };
        let age = safe_age(row.get("seen_pos"));
        if row.get("seen_pos").is_some() && age.is_none() {
            continue;
        }
        let (position, time) = match (position, age) {
            (Some(p), Some(s)) if s <= 15.0 && s <= now => (Some(p), Some(now - s)),
            _ => (None, None),
        };
        let kind = match row.get("type").and_then(Value::as_str).unwrap_or("") {
            "adsb_icao" | "adsb_icao_nt" | "adsb_other" => "adsb",
            "mlat" => "mlat",
            "tisb_icao" | "tisb_other" | "tisb_trackfile" => "tisb",
            "adsr_icao" | "adsr_other" => "rebroadcast",
            _ => "unknown",
        };
        result.push(Observation {
            receiver_id: receiver_id.to_owned(),
            transport: "readsb_json".to_owned(),
            aircraft_key: key,
            received_at_utc: now,
            message_at_utc: now - seen,
            time_basis: if time.is_some() {
                "derived_readsb"
            } else {
                "unknown"
            }
            .to_owned(),
            position_origin: kind.to_owned(),
            position_at_utc: time,
            position,
            callsign: row
                .get("flight")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty() && s.len() <= 8)
                .map(str::to_owned),
            squawk,
        });
    }
    Ok(result)
}

pub fn beast_arrival(hex: &str, timestamp: f64, position: Position) -> Option<Observation> {
    if !timestamp.is_finite()
        || timestamp < 0.0
        || !(-90.0..=90.0).contains(&position.lat)
        || !(-180.0..=180.0).contains(&position.lon)
    {
        return None;
    }
    Some(Observation {
        receiver_id: "rx-primary".to_owned(),
        transport: "beast_tcp".to_owned(),
        aircraft_key: aircraft_key(hex)?,
        received_at_utc: timestamp,
        message_at_utc: timestamp,
        time_basis: "server_arrival".to_owned(),
        position_origin: "adsb".to_owned(),
        position_at_utc: Some(timestamp),
        position: Some(position),
        callsign: None,
        squawk: None,
    })
}

pub fn eligible_for_sql(o: &Observation) -> bool {
    o.position.is_some()
        && o.position_at_utc.is_some()
        && o.position_origin == "adsb"
        && matches!(o.time_basis.as_str(), "derived_readsb" | "receiver_utc")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicAircraft {
    pub id: String,
    pub last_message_at: f64,
    pub last_position_at: Option<f64>,
    pub position: Option<Position>,
    pub position_origin: String,
    pub stale: bool,
    pub callsign: Option<String>,
    pub squawk: Option<String>,
}

pub fn merge(state: &mut BTreeMap<String, PublicAircraft>, o: &Observation) -> &'static str {
    let item = state
        .entry(o.aircraft_key.clone())
        .or_insert(PublicAircraft {
            id: o.aircraft_key.clone(),
            last_message_at: o.message_at_utc,
            last_position_at: None,
            position: None,
            position_origin: o.position_origin.clone(),
            stale: true,
            callsign: None,
            squawk: None,
        });
    item.last_message_at = item.last_message_at.max(o.message_at_utc);
    if let Some(name) = &o.callsign {
        item.callsign = Some(name.clone());
    }
    if let Some(squawk) = &o.squawk {
        item.squawk = Some(squawk.clone());
    }
    if let (Some(next), Some(time)) = (&o.position, o.position_at_utc) {
        if let Some(previous) = item.last_position_at {
            if time <= previous {
                return "duplicate_or_old";
            }
            if let Some(p) = &item.position {
                let dx =
                    (next.lon - p.lon).to_radians() * ((next.lat + p.lat) * 0.5).to_radians().cos();
                let dy = (next.lat - p.lat).to_radians();
                let velocity = 6_371_000.0 * (dx * dx + dy * dy).sqrt() / (time - previous);
                if velocity > 1000.0 {
                    return "conflict";
                }
            }
        }
        item.position = Some(next.clone());
        item.last_position_at = Some(time);
        item.position_origin = o.position_origin.clone();
        item.stale = false;
        "updated"
    } else {
        "no_new_position"
    }
}

/// CRC24 pouze pro DF s nezakrytou paritou; AP/Mode A/C se takto nevalidují.
pub fn crc24(data: &[u8]) -> u32 {
    let mut rem = 0u32;
    for b in data.iter().copied().chain([0u8, 0, 0]) {
        for shift in (0..8).rev() {
            let top = rem & 0x80_0000 != 0;
            rem = ((rem << 1) & 0xff_ffff) | u32::from((b >> shift) & 1);
            if top {
                rem ^= 0xff_f409;
            }
        }
    }
    rem
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, &'static str> {
    if s.len() % 2 != 0 || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("hex");
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).map_err(|_| "utf8")?, 16)
                .map_err(|_| "hex")
        })
        .collect()
}

/// Jen kontrola úplného golden rámce; TCP stavový parser patří G-02.
pub fn beast_envelope(s: &str) -> Result<(u8, u64, u8, Vec<u8>), &'static str> {
    let encoded = hex_decode(s)?;
    if encoded.first() != Some(&0x1a) {
        return Err("prefix");
    }
    let kind = *encoded.get(1).ok_or("type")?;
    let payload_len = match kind {
        0x31 => 2,
        0x32 => 7,
        0x33 => 14,
        _ => return Err("unsupported"),
    };
    let mut bytes = Vec::new();
    let mut i = 2;
    while i < encoded.len() {
        let byte = encoded[i];
        if byte == 0x1a {
            if encoded.get(i + 1) != Some(&0x1a) {
                return Err("escape");
            }
            i += 1;
        }
        bytes.push(byte);
        i += 1;
    }
    if bytes.len() != payload_len + 7 {
        return Err("length");
    }
    let ticks = bytes[..6].iter().fold(0u64, |n, &b| n * 256 + u64::from(b));
    Ok((kind, ticks, bytes[6], bytes[7..].to_vec()))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum StreamEvent {
    Snapshot {
        epoch: String,
        seq: u64,
        #[serde(rename = "generatedAt")]
        generated_at: String,
        aircraft: Vec<PublicAircraft>,
    },
    Delta {
        epoch: String,
        seq: u64,
        #[serde(rename = "generatedAt")]
        generated_at: String,
        upsert: Vec<PublicAircraft>,
        removed: Vec<String>,
    },
}

#[derive(Default)]
pub struct Replica {
    pub epoch: Option<String>,
    pub seq: Option<u64>,
    pub aircraft: BTreeMap<String, PublicAircraft>,
}

impl Replica {
    pub fn apply(&mut self, event: StreamEvent) -> Result<(), &'static str> {
        match event {
            StreamEvent::Snapshot {
                epoch,
                seq,
                aircraft,
                ..
            } => {
                let mut next = BTreeMap::new();
                for item in aircraft {
                    if next.insert(item.id.clone(), item).is_some() {
                        return Err("duplicate");
                    }
                }
                self.aircraft = next;
                self.epoch = Some(epoch);
                self.seq = Some(seq);
            }
            StreamEvent::Delta {
                epoch,
                seq,
                upsert,
                removed,
                ..
            } => {
                if self.epoch.as_deref() != Some(epoch.as_str())
                    || self.seq.and_then(|s| s.checked_add(1)) != Some(seq)
                {
                    return Err("resync required");
                }
                let mut next = self.aircraft.clone();
                for item in upsert {
                    next.insert(item.id.clone(), item);
                }
                for id in removed {
                    next.remove(&id);
                }
                self.aircraft = next;
                self.seq = Some(seq);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn readsb_golden_time_origin_and_nonicao() {
        let f = fixture(include_str!(
            "../../../contracts/fixtures/readsb/initial.json"
        ));
        let expected = fixture(include_str!(
            "../../../contracts/fixtures/readsb/expected.json"
        ));
        let obs = normalize_readsb(&f, "rx-primary", 1792627200.0).unwrap();
        assert_eq!(obs.len(), 3);
        for (actual, expected) in obs.iter().zip(expected["initial"].as_array().unwrap()) {
            let mut normalized = serde_json::to_value(actual).unwrap();
            assert_eq!(normalized["receivedAtUtc"].as_f64(), expected["receivedAtUtc"].as_f64());
            assert_eq!(normalized["messageAtUtc"].as_f64(), expected["messageAtUtc"].as_f64());
            normalized["receivedAtUtc"] = expected["receivedAtUtc"].clone();
            normalized["messageAtUtc"] = expected["messageAtUtc"].clone();
            assert_eq!(normalized, *expected);
        }
        assert_ne!(obs[0].aircraft_key, obs[1].aircraft_key);
        assert!(eligible_for_sql(&obs[0]));
        assert!(!eligible_for_sql(&obs[1]));
        assert!(!eligible_for_sql(&obs[2]));
        let repeated = fixture(include_str!(
            "../../../contracts/fixtures/readsb/repeated.json"
        ));
        let repeat = normalize_readsb(&repeated, "rx-primary", 1792627201.0).unwrap();
        assert_eq!(obs[0].position_at_utc, repeat[0].position_at_utc);
    }

    #[test]
    fn invalid_rows_skip_and_clock_drift_rejects() {
        let bad = fixture(include_str!(
            "../../../contracts/fixtures/readsb/invalid.json"
        ));
        assert!(
            normalize_readsb(&bad, "rx-primary", 1792627200.0)
                .unwrap()
                .is_empty()
        );
        assert!(normalize_readsb(&bad, "rx-primary", 1792627300.0).is_err());
        assert!(normalize_readsb(&bad, "unknown-station", 1792627200.0).is_err());
        let empty = fixture(include_str!(
            "../../../contracts/fixtures/readsb/empty.json"
        ));
        assert!(
            normalize_readsb(&empty, "rx-primary", 1792627210.0)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn beast_without_utc_is_not_sql_history() {
        let obs = beast_arrival(
            "ABC123",
            1792627205.0,
            Position {
                lat: 49.9,
                lon: 15.3,
            },
        )
        .unwrap();
        assert_eq!(obs.time_basis, "server_arrival");
        assert_eq!(obs.aircraft_key, "icao:abc123");
        assert!(!eligible_for_sql(&obs));
    }

    #[test]
    fn merge_dedup_and_plausibility_guard() {
        let f = fixture(include_str!(
            "../../../contracts/fixtures/readsb/initial.json"
        ));
        let repeated = fixture(include_str!(
            "../../../contracts/fixtures/readsb/repeated.json"
        ));
        let changed = fixture(include_str!(
            "../../../contracts/fixtures/readsb/changed.json"
        ));
        let mut state = BTreeMap::new();
        let first = normalize_readsb(&f, "rx-primary", 1792627200.0).unwrap();
        assert_eq!(merge(&mut state, &first[0]), "updated");
        let dup = normalize_readsb(&repeated, "rx-primary", 1792627201.0).unwrap();
        assert_eq!(merge(&mut state, &dup[0]), "duplicate_or_old");
        let fresh = normalize_readsb(&changed, "rx-primary", 1792627205.0).unwrap();
        assert_eq!(merge(&mut state, &fresh[0]), "updated");
        let mut impossible = fresh[0].clone();
        impossible.position_at_utc = Some(1792627205.0);
        impossible.position = Some(Position {
            lat: -20.0,
            lon: 140.0,
        });
        assert_eq!(merge(&mut state, &impossible), "conflict");
        assert_eq!(state["icao:abc123"].position.as_ref().unwrap().lat, 49.91);
    }

    #[test]
    fn beast_golden_frames_crc_escaping_and_cpr_bits() {
        let f = fixture(include_str!("../../../contracts/fixtures/beast/cases.json"));
        for case in f["valid"].as_array().unwrap() {
            let (kind, ticks, rssi, data) = beast_envelope(case["hex"].as_str().unwrap()).unwrap();
            assert_eq!(
                kind,
                u8::from_str_radix(case["type"].as_str().unwrap(), 16).unwrap()
            );
            assert_eq!(ticks, case["receiverTicks"].as_u64().unwrap());
            assert_eq!(rssi, case["rssi"].as_u64().unwrap() as u8);
            assert_eq!(data, hex_decode(case["modeS"].as_str().unwrap()).unwrap());
            if kind == 0x33 {
                assert_eq!(data[0] >> 3, 17);
                assert_eq!(
                    crc24(&data[..11]),
                    u32::from_be_bytes([0, data[11], data[12], data[13]])
                );
                assert_eq!((data[6] >> 2) & 1, if case["cpr"] == "odd" { 1 } else { 0 });
            }
        }
        assert_eq!(
            crc24(&hex_decode(f["knownCrc"]["data"].as_str().unwrap()).unwrap()),
            0x2863a7
        );
        for case in f["invalid"].as_array().unwrap() {
            let hex = case["hex"].as_str().unwrap();
            if case["expected"] == "bad_crc" {
                let (_, _, _, data) = beast_envelope(hex).unwrap();
                assert_ne!(
                    crc24(&data[..11]),
                    u32::from_be_bytes([0, data[11], data[12], data[13]])
                );
            } else {
                assert!(beast_envelope(hex).is_err());
            }
        }
    }

    #[test]
    fn sse_snapshot_delta_gap_and_restart() {
        let f = fixture(include_str!(
            "../../../contracts/fixtures/sse/sequence.json"
        ));
        let mut replica = Replica::default();
        for key in ["snapshot", "delta", "removed"] {
            replica
                .apply(serde_json::from_value(f[key].clone()).unwrap())
                .unwrap();
        }
        assert!(replica.aircraft.is_empty());
        assert!(
            replica
                .apply(serde_json::from_value(f["gap"].clone()).unwrap())
                .is_err()
        );
        assert_eq!(replica.seq, Some(7));
        replica
            .apply(serde_json::from_value(f["restart"].clone()).unwrap())
            .unwrap();
        assert_eq!(replica.seq, Some(0));
    }
}
