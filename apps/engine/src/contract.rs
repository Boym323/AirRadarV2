//! G-01: čisté validační kontrakty, nikoli živý dekodér G-02.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    ReadsbJson,
    BeastTcp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeBasis {
    DerivedReadsb,
    ReceiverUtc,
    ServerArrival,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionOrigin {
    Adsb,
    Mlat,
    Tisb,
    Estimated,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Observation {
    pub receiver_id: String,
    pub transport: Transport,
    pub aircraft_key: String,
    pub received_at_ms: i64,
    pub position_at_ms: Option<i64>,
    pub time_basis: TimeBasis,
    pub position_origin: PositionOrigin,
    pub position: Option<Position>,
    pub callsign: Option<String>,
    pub squawk: Option<String>,
}

pub fn aircraft_key(hex: &str) -> Option<String> {
    let (kind, digits) = if let Some(d) = hex.strip_prefix('~') {
        ("nonicao", d)
    } else {
        ("icao", hex)
    };
    (digits.len() == 6 && digits.bytes().all(|c| c.is_ascii_hexdigit()))
        .then(|| format!("{kind}:{}", digits.to_ascii_lowercase()))
}

pub fn valid(ob: &Observation) -> bool {
    let parts: Vec<&str> = ob.aircraft_key.split(':').collect();
    if parts.len() != 2
        || !matches!(parts[0], "icao" | "nonicao")
        || parts[1].len() != 6
        || !parts[1]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || ob.receiver_id.is_empty()
        || ob.receiver_id.len() > 64
        || !ob
            .receiver_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        || ob.received_at_ms < 0
    {
        return false;
    }
    if let Some(c) = &ob.callsign {
        if c.len() > 16 || c.chars().any(char::is_control) {
            return false;
        }
    }
    if let Some(s) = &ob.squawk {
        if s.len() != 4 || !s.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
            return false;
        }
    }
    match (&ob.position, ob.position_at_ms) {
        (Some(p), Some(t)) => {
            p.lat.is_finite()
                && p.lon.is_finite()
                && (-90.0..=90.0).contains(&p.lat)
                && (-180.0..=180.0).contains(&p.lon)
                && t >= 0
                && t <= ob.received_at_ms + 10_000
                && ob.time_basis != TimeBasis::Unknown
        }
        (None, None) => true,
        _ => false,
    }
}

/// BEAST časové tiky samy o sobě nezakládají UTC. Server arrival nesmí
/// předstírat potvrzený čas polohy určený pro SQL.
pub fn persistable(ob: &Observation) -> bool {
    valid(ob)
        && ob.position.is_some()
        && matches!(
            ob.time_basis,
            TimeBasis::DerivedReadsb | TimeBasis::ReceiverUtc
        )
        && ob.position_origin == PositionOrigin::Adsb
}

/// První shodná identita je pouze kandidát: nemožnou rychlost mezi pozicemi
/// nezaměnit za validní přepsání. Limit slouží jako první bezpečnostní filtr.
pub fn plausible_transition(previous: &Observation, next: &Observation) -> bool {
    if !valid(previous) || !valid(next) || previous.aircraft_key != next.aircraft_key {
        return false;
    }
    match (
        &previous.position,
        &next.position,
        previous.position_at_ms,
        next.position_at_ms,
    ) {
        (Some(a), Some(b), Some(t0), Some(t1)) => {
            if t1 < t0 {
                return false;
            }
            if t1 == t0 {
                return a == b;
            }
            let lat_km = (a.lat - b.lat) * 111.2;
            let lon_km = (a.lon - b.lon) * 111.2 * ((a.lat + b.lat).to_radians() / 2.0).cos();
            let distance_km = lat_km.hypot(lon_km);
            distance_km * 3_600_000.0 / (t1 - t0) as f64 <= 1500.0
        }
        _ => true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PublicAircraft {
    pub aircraft_key: String,
    pub position: Option<Position>,
    pub position_at_ms: Option<i64>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamPayload {
    Snapshot {
        epoch: String,
        seq: u64,
        aircraft: Vec<PublicAircraft>,
    },
    Delta {
        epoch: String,
        seq: u64,
        upsert: Vec<PublicAircraft>,
        removed: Vec<String>,
    },
}

/// Záruka proti mezeře: po registraci odběratele musí snapshot zachytit
/// stejnou nebo novější sekvenci než eventy a receiver nesmí ztratit mezikrok.
pub fn accepts_delta(epoch: &str, last_seq: u64, delta_epoch: &str, next_seq: u64) -> bool {
    epoch == delta_epoch && last_seq.checked_add(1) == Some(next_seq)
}

#[cfg(test)]
mod tests {
    use super::*;
    const JSON: &str = include_str!("../../../contracts/fixtures/observations.json");
    #[test]
    fn shared_golden_cases() {
        let fixtures: serde_json::Value = serde_json::from_str(JSON).unwrap();
        for case in fixtures["cases"].as_array().unwrap() {
            let ob: Observation = serde_json::from_value(case["observation"].clone()).unwrap();
            assert_eq!(
                valid(&ob),
                case["valid"].as_bool().unwrap(),
                "{:?}",
                case["name"]
            );
            assert_eq!(
                persistable(&ob),
                case["persistable"].as_bool().unwrap(),
                "{:?}",
                case["name"]
            );
        }
    }
    #[test]
    fn identity_scope_and_transition() {
        assert_eq!(aircraft_key("ABC123").as_deref(), Some("icao:abc123"));
        assert_eq!(aircraft_key("~ABC123").as_deref(), Some("nonicao:abc123"));
        assert_eq!(aircraft_key("xxx"), None);
        let mut a: Observation = serde_json::from_value(
            serde_json::from_str::<serde_json::Value>(JSON).unwrap()["cases"][0]["observation"]
                .clone(),
        )
        .unwrap();
        let mut b = a.clone();
        b.received_at_ms += 1000;
        b.position_at_ms = Some(a.position_at_ms.unwrap() + 1000);
        b.position.as_mut().unwrap().lat = -40.0;
        assert!(!plausible_transition(&a, &b));
        a.transport = Transport::BeastTcp;
        assert_eq!(a.aircraft_key, b.aircraft_key);
    }
    #[test]
    fn sse_gap_and_restart() {
        assert!(accepts_delta("engine-a", 7, "engine-a", 8));
        assert!(!accepts_delta("engine-a", 7, "engine-a", 9));
        assert!(!accepts_delta("engine-a", 7, "engine-b", 8));
        assert!(!accepts_delta("engine-a", u64::MAX, "engine-a", 0));
    }
}
