//! G-02.06/07/09: jediný vlastník kanonického AircraftState, žádné SQL.
use std::collections::BTreeMap;
use crate::contract::{Observation, PublicAircraft, merge};

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
    pub counters: Counters,
}
impl RadarState {
    pub fn apply(&mut self, observation: &Observation) -> &'static str {
        self.counters.observations += 1;
        if !self.tracks.contains_key(&observation.aircraft_key) && self.tracks.len() >= MAX_AIRCRAFT {
            self.counters.capacity_drops += 1;
            return "capacity";
        }
        let result = merge(&mut self.tracks, observation);
        match result {
            "updated" => self.counters.updated += 1,
            "duplicate_or_old" => self.counters.duplicates += 1,
            "conflict" => self.counters.conflicts += 1,
            _ => {}
        }
        result
    }
    pub fn batch(&mut self, items: &[Observation]) {
        for item in items { self.apply(item); }
    }
    pub fn cleanup(&mut self, now: f64) {
        let before = self.tracks.len();
        self.tracks.retain(|_, p| now - p.last_message_at <= TRACK_TTL_SECONDS);
        self.counters.expired += (before - self.tracks.len()) as u64;
    }
    pub fn snapshot(&self, now: f64) -> Vec<PublicAircraft> {
        self.tracks.values().map(|p| {
            let mut result = p.clone();
            result.stale = p.last_position_at.is_none_or(|t| now - t > POSITION_FRESH_SECONDS);
            result
        }).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::json::parse_replay;
    #[test]
    fn replay_dedup_and_ttl() {
        let first=parse_replay(include_bytes!("../../../../contracts/fixtures/readsb/initial.json"),"rx-primary",1792627200.0).unwrap();
        let second=parse_replay(include_bytes!("../../../../contracts/fixtures/readsb/repeated.json"),"rx-primary",1792627201.0).unwrap();
        let changed=parse_replay(include_bytes!("../../../../contracts/fixtures/readsb/changed.json"),"rx-primary",1792627205.0).unwrap();
        let mut state=RadarState::default();
        state.batch(&first);
        assert_eq!(state.tracks.len(),3);
        state.batch(&second);
        assert_eq!(state.tracks.len(),3);
        state.batch(&changed);
        assert_eq!(state.tracks.len(),3);
        assert_eq!(state.tracks["icao:abc123"].position.as_ref().unwrap().lat,49.91);
        assert_eq!(state.snapshot(1792627222.0).iter().find(|x|x.id=="icao:abc123").unwrap().stale,true);
        state.cleanup(1792627300.0);
        assert!(state.tracks.is_empty());
        assert_eq!(state.counters.expired,3);
    }
    #[test]
    fn cap_five_thousand() {
        let mut state=RadarState::default();
        for n in 0..=MAX_AIRCRAFT {
            let data=Observation {
                receiver_id:"rx-primary".into(),transport:"readsb_json".into(),
                aircraft_key:format!("icao:{n:06x}"),
                received_at_utc:100.0,message_at_utc:100.0,
                time_basis:"unknown".into(),position_origin:"adsb".into(),
                position_at_utc:None,position:None,callsign:None,squawk:None,
            };
            state.apply(&data);
        }
        assert_eq!(state.tracks.len(), MAX_AIRCRAFT);
        assert_eq!(state.counters.capacity_drops,1);
    }
}
