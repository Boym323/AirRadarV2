//! G-02.06/07/09: jediný vlastník kanonického AircraftState, žádné SQL.
use crate::contract::{FlightData, Observation, Position, PublicAircraft, merge};
use std::collections::BTreeMap;

pub const MAX_AIRCRAFT: usize = 5000;
pub const TRACK_TTL_SECONDS: f64 = 60.0;
pub const POSITION_FRESH_SECONDS: f64 = 15.0;

#[derive(Default, Debug)]
pub struct Counters {
    pub observations: u64,
    pub updated: u64,
    pub duplicates: u64,
    pub conflicts: u64,
    pub capacity_drops: u64,
    pub expired: u64,
}
#[derive(Default)]
pub struct RadarState {
    pub tracks: BTreeMap<String, PublicAircraft>,
    /// Interní kvalita časové základny poslední polohy (jen RAM).
    position_quality: BTreeMap<String, u8>,
    pub counters: Counters,
}
impl RadarState {
    pub fn apply(&mut self, observation: &Observation) -> &'static str {
        self.counters.observations += 1;
        if observation.receiver_id != "rx-primary"
            || !observation.message_at_utc.is_finite()
            || !observation.received_at_utc.is_finite()
            || observation.message_at_utc > observation.received_at_utc + 10.0
            || observation
                .position_at_utc
                .is_some_and(|t| !t.is_finite() || t < 0.0)
        {
            return "invalid";
        }
        if !self.tracks.contains_key(&observation.aircraft_key) && self.tracks.len() >= MAX_AIRCRAFT
        {
            self.counters.capacity_drops += 1;
            return "capacity";
        }
        let incoming_quality = match observation.time_basis.as_str() {
            "receiver_utc" | "derived_readsb" => 2,
            "server_arrival" => 1,
            _ => 0,
        };
        // Nižší časová přesnost nesmí krátce po důvěryhodném JSON
        // přepsat ani čerstvou polohu, ani metadata. Jinak by novější
        // server_arrival u BEAST přepsal kvalitnější readsb hodnoty.
        let masked = self
            .tracks
            .get(&observation.aircraft_key)
            .and_then(|old| old.last_position_at.map(|time| (old, time)))
            .is_some_and(|(_, t)| {
                self.position_quality
                    .get(&observation.aircraft_key)
                    .copied()
                    .unwrap_or(0)
                    > incoming_quality
                    && observation.received_at_utc - t <= POSITION_FRESH_SECONDS
            });
        let mut incoming = observation.clone();
        if masked {
            incoming.position = None;
            incoming.position_at_utc = None;
            incoming.callsign = None;
            incoming.squawk = None;
            incoming.telemetry = FlightData::default();
        }
        let result = merge(&mut self.tracks, &incoming);
        match result {
            "updated" => {
                self.counters.updated += 1;
                self.position_quality
                    .insert(observation.aircraft_key.clone(), incoming_quality);
            }
            "duplicate_or_old" => self.counters.duplicates += 1,
            "conflict" => self.counters.conflicts += 1,
            _ => {}
        }
        result
    }
    pub fn batch(&mut self, items: &[Observation]) {
        for item in items {
            self.apply(item);
        }
    }
    pub fn cleanup(&mut self, now: f64) {
        let before = self.tracks.len();
        self.tracks
            .retain(|_, p| now - p.last_message_at <= TRACK_TTL_SECONDS);
        self.position_quality
            .retain(|key, _| self.tracks.contains_key(key));
        self.counters.expired += (before - self.tracks.len()) as u64;
    }
    /// Důvěryhodná čerstvá JSON poloha slouží jako kotva pro lokální CPR.
    pub fn trusted_reference(&self, key: &str, now: f64) -> Option<Position> {
        let aircraft = self.tracks.get(key)?;
        if self.position_quality.get(key).copied().unwrap_or(0) < 2
            || now - aircraft.last_position_at? > POSITION_FRESH_SECONDS
        {
            return None;
        }
        aircraft.position.clone()
    }
    pub fn snapshot(&self, now: f64) -> Vec<PublicAircraft> {
        self.tracks
            .values()
            .map(|p| {
                let mut result = p.clone();
                result.stale = p
                    .last_position_at
                    .is_none_or(|t| now - t > POSITION_FRESH_SECONDS);
                result
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::FlightData;
    use crate::ingest::json::parse_replay;
    #[test]
    fn replay_dedup_and_ttl() {
        let first = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap();
        let second = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/repeated.json"),
            "rx-primary",
            1792627201.0,
        )
        .unwrap();
        let changed = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/changed.json"),
            "rx-primary",
            1792627205.0,
        )
        .unwrap();
        let mut state = RadarState::default();
        state.batch(&first);
        assert_eq!(state.tracks.len(), 3);
        assert_eq!(
            state.tracks["icao:abc123"].telemetry.altitude_baro_ft,
            Some(10000)
        );
        assert_eq!(
            state.tracks["icao:abc123"].telemetry.ground_speed_kt,
            Some(180.0)
        );
        state.batch(&second);
        assert_eq!(state.tracks.len(), 3);
        state.batch(&changed);
        assert_eq!(state.tracks.len(), 3);
        assert_eq!(
            state.tracks["icao:abc123"].position.as_ref().unwrap().lat,
            49.91
        );
        assert!(
            state
                .snapshot(1792627222.0)
                .iter()
                .find(|x| x.id == "icao:abc123")
                .unwrap()
                .stale
        );
        state.cleanup(1792627300.0);
        assert!(state.tracks.is_empty());
        assert!(
            state.position_quality.is_empty(),
            "Pomocná RAM mapa nesmí leakovat"
        );
        assert_eq!(state.counters.expired, 3);
        assert!(
            state
                .trusted_reference("icao:abc123", 1792627300.0)
                .is_none()
        );
    }
    #[test]
    fn cap_five_thousand() {
        let mut state = RadarState::default();
        for n in 0..=MAX_AIRCRAFT {
            let data = Observation {
                receiver_id: "rx-primary".into(),
                transport: "readsb_json".into(),
                aircraft_key: format!("icao:{n:06x}"),
                received_at_utc: 100.0,
                message_at_utc: 100.0,
                time_basis: "unknown".into(),
                position_origin: "adsb".into(),
                position_at_utc: None,
                position: None,
                callsign: None,
                squawk: None,
                telemetry: FlightData::default(),
            };
            state.apply(&data);
        }
        assert_eq!(state.tracks.len(), MAX_AIRCRAFT);
        assert_eq!(state.counters.capacity_drops, 1);
    }
    #[test]
    fn beast_cannot_override_fresh_trusted_json() {
        let first = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap();
        let mut state = RadarState::default();
        state.batch(&first);
        let mut beast = first[0].clone();
        beast.transport = "beast_tcp".into();
        beast.time_basis = "server_arrival".into();
        beast.received_at_utc = 1792627203.0;
        beast.message_at_utc = 1792627203.0;
        beast.position_at_utc = Some(1792627203.0);
        beast.position.as_mut().unwrap().lat = 49.901;
        beast.callsign = Some("SPOOF".into());
        beast.squawk = Some("7777".into());
        beast.telemetry.ground_speed_kt = Some(999.0);
        state.apply(&beast);
        assert_eq!(state.tracks["icao:abc123"].callsign.as_deref(), Some("TEST123"));
        assert_eq!(state.tracks["icao:abc123"].squawk.as_deref(), Some("7600"));
        assert_eq!(state.tracks["icao:abc123"].telemetry.ground_speed_kt, Some(180.0));
        assert_eq!(
            state.tracks["icao:abc123"].position.as_ref().unwrap().lat,
            49.9
        );
        beast.received_at_utc = 1792627220.0;
        beast.message_at_utc = 1792627220.0;
        beast.position_at_utc = Some(1792627220.0);
        assert_eq!(state.apply(&beast), "updated");
        // Po vypršení důvěryhodné JSON freshness může převzít údaje BEAST.
        assert_eq!(state.tracks["icao:abc123"].callsign.as_deref(), Some("SPOOF"));
        assert_eq!(state.tracks["icao:abc123"].telemetry.ground_speed_kt, Some(999.0));
        assert_eq!(
            state.tracks["icao:abc123"].position.as_ref().unwrap().lat,
            49.901
        );
    }
    #[test]
    fn implausible_position_does_not_mutate_metadata() {
        let first = parse_replay(
            include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),
            "rx-primary",
            1792627200.0,
        )
        .unwrap();
        let mut state = RadarState::default();
        state.apply(&first[0]);
        let mut bad = first[0].clone();
        bad.received_at_utc = 1792627203.0;
        bad.message_at_utc = 1792627203.0;
        bad.position_at_utc = Some(1792627203.0);
        bad.position.as_mut().unwrap().lat = -70.0;
        bad.callsign = Some("SPOOF".into());
        assert_eq!(state.apply(&bad), "conflict");
        assert_eq!(
            state.tracks["icao:abc123"].callsign.as_deref(),
            Some("TEST123")
        );
    }
}
