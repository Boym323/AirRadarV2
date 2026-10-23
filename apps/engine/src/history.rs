//! G-04.06/.07 · J09: RAM-first výběr historických bodů bez síťového I/O.
//! Jediným producentem je autoritativní ingest actor; SSE ani parser nečekají na SQL.
use crate::contract::{Observation, eligible_for_sql};
use std::collections::BTreeMap;

pub const MAX_PENDING: usize = 2048;
pub const MAX_BATCH: usize = 128;
pub const MAX_TRACK_WATERMARKS: usize = 5000;
const BUCKET_SECONDS: i64 = 30;
const CLOSE_DELAY: f64 = 5.0;
const MAX_AFTER_CLOSE: f64 = 30.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub bucket_at: i64,
    pub aircraft_key: String,
    pub position_at: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_ft: Option<i32>,
    pub ground_speed_kt: Option<f64>,
    pub track_deg: Option<f64>,
    pub squawk: Option<String>,
    pub on_ground: Option<bool>,
}

impl Sample {
    fn from_observation(o: &Observation) -> Option<Self> {
        if !eligible_for_sql(o) || o.receiver_id != "rx-primary" {
            return None;
        }
        let p = o.position.as_ref()?;
        let at = o.position_at_utc?;
        if !at.is_finite()
            || at <= 0.0
            || at > o.received_at_utc + 2.0
            || o.received_at_utc - at > 15.0
            || !p.lat.is_finite()
            || !p.lon.is_finite()
            || !(-90.0..=90.0).contains(&p.lat)
            || !(-180.0..=180.0).contains(&p.lon)
        {
            return None;
        }
        let suffix = o
            .aircraft_key
            .strip_prefix("icao:")
            .or_else(|| o.aircraft_key.strip_prefix("nonicao:"))?;
        if suffix.len() != 6
            || !suffix
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return None;
        }
        let bucket_at = (at / BUCKET_SECONDS as f64).floor() as i64 * BUCKET_SECONDS;
        Some(Self {
            bucket_at,
            aircraft_key: o.aircraft_key.clone(),
            position_at: at,
            latitude: p.lat,
            longitude: p.lon,
            altitude_ft: o.telemetry.altitude_baro_ft,
            ground_speed_kt: o.telemetry.ground_speed_kt,
            track_deg: o.telemetry.track_degrees,
            squawk: o.squawk.clone(),
            on_ground: o.telemetry.on_ground,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct HistoryCounters {
    pub eligible: u64,
    pub skipped: u64,
    pub coalesced: u64,
    pub dropped_capacity: u64,
    pub dropped_stale: u64,
    pub dropped_writer: u64,
    pub committed: u64,
    pub batches: u64,
}

#[derive(Default)]
pub struct HistorySampler {
    // Všechny mapy mají pevný limit; watermarks se odstraňují při stárnutí.
    confirmed: BTreeMap<String, Sample>,
    pending: BTreeMap<(i64, String), Sample>,
    in_flight: Vec<Sample>,
    pub counters: HistoryCounters,
}

fn distance_m(a: &Sample, b: &Sample) -> f64 {
    let lat = ((a.latitude + b.latitude) * 0.5).to_radians();
    let dx = (b.longitude - a.longitude).to_radians() * lat.cos();
    let dy = (b.latitude - a.latitude).to_radians();
    6_371_000.0 * (dx * dx + dy * dy).sqrt()
}

fn heading_delta(a: Option<f64>, b: Option<f64>) -> f64 {
    match (a, b) {
        (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
            let diff = (a - b).abs().rem_euclid(360.0);
            diff.min(360.0 - diff)
        }
        _ => 0.0,
    }
}

fn worth_sampling(last: &Sample, next: &Sample) -> bool {
    let dt = next.position_at - last.position_at;
    if dt <= 0.0 {
        return false;
    }
    let distance = distance_m(last, next);
    let heading = heading_delta(last.track_deg, next.track_deg);
    let altitude = match (last.altitude_ft, next.altitude_ft) {
        (Some(a), Some(b)) => (i64::from(a) - i64::from(b)).abs(),
        _ => 0,
    };
    if last.on_ground == Some(true) && next.on_ground == Some(true) {
        return (dt >= 120.0 && distance >= 100.0) || dt >= 600.0;
    }
    let transition =
        last.on_ground.is_some() && next.on_ground.is_some() && last.on_ground != next.on_ground;
    if dt >= 30.0 && (transition || heading >= 30.0 || altitude >= 1500) {
        return true;
    }
    dt >= 180.0 || (dt >= 60.0 && (distance >= 2000.0 || heading >= 15.0 || altitude >= 500))
}

impl HistorySampler {
    pub fn observe(&mut self, o: &Observation) -> bool {
        let Some(sample) = Sample::from_observation(o) else {
            self.counters.skipped += 1;
            return false;
        };
        let key = (sample.bucket_at, sample.aircraft_key.clone());
        // Zlepšený bod v dosud otevřeném UTC bucketu se sloučí v RAM,
        // bez SQL UPDATE a bez zvýšení počtu SQL pokusů.
        if let Some(existing) = self.pending.get_mut(&key) {
            if sample.position_at > existing.position_at {
                *existing = sample;
                self.counters.coalesced += 1;
                return true;
            }
            self.counters.skipped += 1;
            return false;
        }
        if self
            .in_flight
            .iter()
            .any(|s| s.bucket_at == sample.bucket_at && s.aircraft_key == sample.aircraft_key)
            || self
                .confirmed
                .get(&sample.aircraft_key)
                .is_some_and(|s| s.bucket_at == sample.bucket_at)
        {
            self.counters.skipped += 1;
            return false;
        }
        let mut baseline = self.confirmed.get(&sample.aircraft_key);
        for entry in self.pending.values().chain(self.in_flight.iter()) {
            if entry.aircraft_key == sample.aircraft_key
                && baseline.is_none_or(|old| entry.position_at > old.position_at)
            {
                baseline = Some(entry);
            }
        }
        if baseline.is_some_and(|last| !worth_sampling(last, &sample)) {
            self.counters.skipped += 1;
            return false;
        }
        if self.pending.len() + self.in_flight.len() >= MAX_PENDING {
            // Odhození nejstaršího neuloženého bucketu, nikoli živého RAM tracku.
            self.pending.pop_first();
            self.counters.dropped_capacity += 1;
        }
        self.pending.insert(key, sample);
        self.counters.eligible += 1;
        true
    }

    /// Potřebuje jen zavřené buckety. Při výpadku SQL má čekání tvrdý limit.
    pub fn next_batch(&mut self, now: f64) -> Vec<Sample> {
        self.expire(now);
        if !self.in_flight.is_empty() || !now.is_finite() {
            return Vec::new();
        }
        let keys: Vec<_> = self
            .pending
            .iter()
            .filter(|((bucket, _), _)| {
                (*bucket as f64 + BUCKET_SECONDS as f64 + CLOSE_DELAY) <= now
            })
            .take(MAX_BATCH)
            .map(|(k, _)| k.clone())
            .collect();
        for key in keys {
            if let Some(value) = self.pending.remove(&key) {
                self.in_flight.push(value);
            }
        }
        self.in_flight.clone()
    }

    /// Volat výhradně po výsledku DB statementu (potvrzený COMMIT nebo chyba).
    pub fn finish_batch(&mut self, committed: bool, now: f64) {
        let count = self.in_flight.len() as u64;
        if count == 0 {
            return;
        }
        if committed {
            for sample in self.in_flight.drain(..) {
                self.confirmed.insert(sample.aircraft_key.clone(), sample);
            }
            self.counters.committed += count;
            self.counters.batches += 1;
        } else {
            self.in_flight.clear();
            self.counters.dropped_writer += count;
        }
        self.prune_confirmed(now);
    }

    pub fn expire(&mut self, now: f64) {
        if !now.is_finite() {
            return;
        }
        let before = self.pending.len();
        self.pending.retain(|(bucket, _), _| {
            now < *bucket as f64 + BUCKET_SECONDS as f64 + MAX_AFTER_CLOSE
        });
        self.counters.dropped_stale += (before - self.pending.len()) as u64;
        self.prune_confirmed(now);
    }

    fn prune_confirmed(&mut self, now: f64) {
        if now.is_finite() {
            self.confirmed.retain(|_, s| s.position_at > now - 900.0);
        }
        while self.confirmed.len() > MAX_TRACK_WATERMARKS {
            let oldest = self
                .confirmed
                .iter()
                .min_by(|a, b| a.1.position_at.total_cmp(&b.1.position_at))
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                self.confirmed.remove(&key);
            }
        }
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len() + self.in_flight.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{FlightData, Position};
    fn observation(t: f64, lon: f64) -> Observation {
        Observation {
            receiver_id: "rx-primary".into(),
            transport: "readsb_json".into(),
            aircraft_key: "icao:abcdef".into(),
            received_at_utc: t,
            message_at_utc: t,
            time_basis: "derived_readsb".into(),
            position_origin: "adsb".into(),
            position_at_utc: Some(t),
            position: Some(Position { lat: 49.0, lon }),
            callsign: None,
            squawk: Some("7000".into()),
            telemetry: FlightData {
                altitude_baro_ft: Some(10000),
                ground_speed_kt: Some(200.0),
                track_degrees: Some(90.0),
                on_ground: Some(false),
                ..Default::default()
            },
        }
    }
    fn commit(s: &mut HistorySampler, t: f64) {
        assert_eq!(s.next_batch(t).len(), 1);
        s.finish_batch(true, t);
    }
    #[test]
    fn first_position_coalesces_only_in_ram() {
        let mut s = HistorySampler::default();
        assert!(s.observe(&observation(1800.0, 17.0)));
        assert!(s.observe(&observation(1801.0, 17.001)));
        assert_eq!(s.pending_len(), 1);
        assert_eq!(s.counters.coalesced, 1);
        assert_eq!(s.next_batch(1834.0).len(), 0);
        commit(&mut s, 1836.0);
        assert_eq!(s.counters.committed, 1);
        assert!(!s.observe(&observation(1802.0, 17.0)));
    }
    #[test]
    fn timing_and_quality_rules() {
        let mut s = HistorySampler::default();
        assert!(s.observe(&observation(1800.0, 17.0)));
        commit(&mut s, 1836.0);
        assert!(!s.observe(&observation(1820.0, 17.06)));
        assert!(!s.observe(&observation(1860.0, 17.001))); // 60s, ne 2km
        assert!(s.observe(&observation(1860.0, 17.04))); // 60s, >2km
        commit(&mut s, 1896.0);
        let mut turn = observation(1890.0, 17.041);
        turn.telemetry.track_degrees = Some(125.0);
        assert!(s.observe(&turn)); // 30 s, velká změna směru
        commit(&mut s, 1926.0);
        let mut stale = observation(1950.0, 17.1);
        stale.position_at_utc = Some(1900.0);
        assert!(!s.observe(&stale));
        stale.time_basis = "server_arrival".into();
        stale.position_at_utc = Some(1950.0);
        assert!(!s.observe(&stale));
        assert!(s.counters.skipped >= 4);
    }
    #[test]
    fn ground_timeout_and_failure() {
        let mut s = HistorySampler::default();
        let mut first = observation(2400.0, 17.0);
        first.telemetry.on_ground = Some(true);
        assert!(s.observe(&first));
        commit(&mut s, 2436.0);
        let mut still = observation(2530.0, 17.0002);
        still.telemetry.on_ground = Some(true);
        assert!(!s.observe(&still));
        still.position_at_utc = Some(3001.0);
        still.received_at_utc = 3001.0;
        assert!(s.observe(&still));
        assert_eq!(s.next_batch(3036.0).len(), 1);
        s.finish_batch(false, 3036.0);
        assert_eq!(s.counters.dropped_writer, 1);
        assert_eq!(s.pending_len(), 0);
    }
    #[test]
    fn bounded_and_expiring_queue_no_replay_storm() {
        let mut s = HistorySampler::default();
        for i in 0..MAX_PENDING + 200 {
            let mut o = observation(1800.0, 17.0);
            o.aircraft_key = format!("icao:{i:06x}");
            s.observe(&o);
        }
        assert_eq!(s.pending_len(), MAX_PENDING);
        assert_eq!(s.counters.dropped_capacity, 200);
        s.expire(1900.0);
        assert_eq!(s.pending_len(), 0);
        assert_eq!(s.counters.dropped_stale, MAX_PENDING as u64);
        assert!(s.next_batch(1900.0).is_empty());
    }
    #[test]
    fn no_sql_on_replayed_transport_or_invalid_position() {
        let mut s = HistorySampler::default();
        assert!(s.observe(&observation(1800.0, 17.0)));
        let mut duplicate = observation(1800.0, 17.0);
        duplicate.transport = "beast_tcp".into();
        duplicate.time_basis = "server_arrival".into();
        assert!(!s.observe(&duplicate));
        let corrupted = observation(1800.0, f64::NAN);
        assert!(!s.observe(&corrupted));
        assert_eq!(s.pending_len(), 1);
    }
}
