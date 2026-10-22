//! G-02.04/05: konzervativní DF17 CRC, callsign a globální airborne CPR.
//! BEAST tick (12 MHz) není UTC; pro živý RAM track používáme přijetí serverem.
use crate::contract::{FlightData, Observation, Position, aircraft_key, crc24};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Cpr {
    pub odd: bool,
    lat: f64,
    lon: f64,
    pub at: f64,
}
#[derive(Clone, Debug)]
pub struct Decoded {
    pub key: String,
    pub callsign: Option<String>,
    pub cpr: Option<Cpr>,
    pub telemetry: FlightData,
}

/// Pouze DF17: DF0/4/5/20/21 mají AP/address-parity a nelze pro ně
/// slepě použít 0 CRC syndrom.
pub fn decode_df17(data: &[u8], received_at: f64) -> Option<Decoded> {
    if data.len() != 14
        || data[0] >> 3 != 17
        || crc24(data) != 0
        || !received_at.is_finite()
        || received_at < 0.0
    {
        return None;
    }
    let key = aircraft_key(&format!("{:02x}{:02x}{:02x}", data[1], data[2], data[3]))?;
    let tc = data[4] >> 3;
    let callsign = if (1..=4).contains(&tc) {
        const CHARS: &[u8] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ#####_###############0123456789######";
        let mut raw = u64::from(data[5]) << 40
            | u64::from(data[6]) << 32
            | u64::from(data[7]) << 24
            | u64::from(data[8]) << 16
            | u64::from(data[9]) << 8
            | u64::from(data[10]);
        let mut name = String::new();
        for _ in 0..8 {
            let index = ((raw >> 42) & 0x3f) as usize;
            name.push(if CHARS[index] == b'#' || CHARS[index] == b'_' {
                ' '
            } else {
                CHARS[index] as char
            });
            raw <<= 6;
        }
        let trimmed = name.trim().to_owned();
        (!trimmed.is_empty()).then_some(trimmed)
    } else {
        None
    };
    // ADS-B airborne barometrická výška: 12bit AC s Q=1 (25 ft kroky).
    // Q=0 vyžaduje Gillham dekodér, proto není bezpečně odhadována.
    let mut telemetry = FlightData::default();
    if (9..=18).contains(&tc) {
        let ac = u16::from(data[5]) << 4 | u16::from(data[6] >> 4);
        if ac & 0x10 != 0 {
            let n = i32::from((ac & 0x0fe0) >> 1 | (ac & 0x000f));
            let altitude = n * 25 - 1000;
            if (-2000..=65000).contains(&altitude) {
                telemetry.altitude_baro_ft = Some(altitude);
                telemetry.on_ground = Some(false);
            }
        }
    }
    if tc == 19 {
        let subtype = data[4] & 7;
        if subtype == 1 || subtype == 2 {
            let me = data[4..11]
                .iter()
                .fold(0u64, |acc, &byte| (acc << 8) | u64::from(byte));
            let bit = |start: u32, len: u32| -> u32 {
                ((me >> (57 - start - len)) & ((1u64 << len) - 1)) as u32
            };
            let east_raw = bit(15, 10);
            let north_raw = bit(26, 10);
            let factor = if subtype == 2 { 4.0 } else { 1.0 };
            if east_raw > 0 && north_raw > 0 {
                let east =
                    f64::from(east_raw - 1) * factor * if bit(14, 1) == 1 { -1.0 } else { 1.0 };
                let north =
                    f64::from(north_raw - 1) * factor * if bit(25, 1) == 1 { -1.0 } else { 1.0 };
                let speed = east.hypot(north);
                if speed <= 1500.0 {
                    telemetry.ground_speed_kt = Some(speed);
                    if speed > 0.0 {
                        telemetry.track_degrees =
                            Some(east.atan2(north).to_degrees().rem_euclid(360.0));
                    }
                }
            }
            let vertical_raw = bit(38, 9);
            if vertical_raw > 0 {
                let magnitude = ((vertical_raw - 1) * 64) as i32;
                if magnitude <= 15000 {
                    telemetry.vertical_rate_fpm = Some(if bit(37, 1) == 1 {
                        -magnitude
                    } else {
                        magnitude
                    });
                }
            }
        }
    }
    let cpr = if (9..=18).contains(&tc) {
        Some(Cpr {
            odd: data[6] & 4 != 0,
            lat: f64::from(
                (u32::from(data[6] & 3) << 15)
                    | (u32::from(data[7]) << 7)
                    | (u32::from(data[8]) >> 1),
            ) / 131_072.0,
            lon: f64::from(
                (u32::from(data[8] & 1) << 16) | (u32::from(data[9]) << 8) | u32::from(data[10]),
            ) / 131_072.0,
            at: received_at,
        })
    } else {
        None
    };
    Some(Decoded {
        key,
        callsign,
        cpr,
        telemetry,
    })
}

fn nl(lat: f64) -> i32 {
    let absolute = lat.abs();
    if absolute < 1.0e-12 {
        return 59;
    }
    if absolute >= 87.0 {
        return 1;
    }
    let coslat = absolute.to_radians().cos();
    let v = 1.0 - (1.0 - (std::f64::consts::PI / 30.0).cos()) / (coslat * coslat);
    if !(-1.0..=1.0).contains(&v) {
        return 1;
    }
    (2.0 * std::f64::consts::PI / v.acos()).floor() as i32
}

/// Airborne globální CPR: párování sudého a lichého rámce stejného letadla
/// do 10 s, bez nebezpečného lokálního dekódování bez trusted reference.
pub fn global_airborne(even: &Cpr, odd: &Cpr) -> Option<Position> {
    if even.odd || !odd.odd || (even.at - odd.at).abs() > 10.0 {
        return None;
    }
    let j = (59.0 * even.lat - 60.0 * odd.lat + 0.5).floor();
    let mut lat0 = 6.0 * (j.rem_euclid(60.0) + even.lat);
    let mut lat1 = (360.0 / 59.0) * (j.rem_euclid(59.0) + odd.lat);
    if lat0 >= 270.0 {
        lat0 -= 360.0;
    }
    if lat1 >= 270.0 {
        lat1 -= 360.0;
    }
    if lat0.abs() > 90.0 || lat1.abs() > 90.0 || nl(lat0) != nl(lat1) {
        return None;
    }
    let use_odd = odd.at > even.at;
    let lat = if use_odd { lat1 } else { lat0 };
    let nli = nl(lat);
    let m = (even.lon * f64::from(nli - 1) - odd.lon * f64::from(nli) + 0.5).floor();
    let ni = (nli - i32::from(use_odd)).max(1);
    let xz = if use_odd { odd.lon } else { even.lon };
    let mut lon = (360.0 / f64::from(ni)) * (m.rem_euclid(f64::from(ni)) + xz);
    if lon >= 180.0 {
        lon -= 360.0;
    }
    Some(Position { lat, lon })
}

/// Lokální CPR pouze kolem již validované a čerstvé polohy STEJNÉHO letadla.
/// Bez referenční polohy se toto dekódování nikdy nespouští.
pub fn local_airborne(cpr: &Cpr, reference: &Position) -> Option<Position> {
    if !reference.lat.is_finite()
        || !reference.lon.is_finite()
        || !(-90.0..=90.0).contains(&reference.lat)
        || !(-180.0..=180.0).contains(&reference.lon)
        || !(0.0..1.0).contains(&cpr.lat)
        || !(0.0..1.0).contains(&cpr.lon)
    {
        return None;
    }
    let dlat = 360.0 / if cpr.odd { 59.0 } else { 60.0 };
    let j = (reference.lat / dlat).floor()
        + (0.5 + reference.lat.rem_euclid(dlat) / dlat - cpr.lat).floor();
    let mut lat = dlat * (j + cpr.lat);
    if lat >= 270.0 {
        lat -= 360.0;
    }
    if !(-90.0..=90.0).contains(&lat) {
        return None;
    }
    let n = (nl(lat) - i32::from(cpr.odd)).max(1);
    let dlon = 360.0 / f64::from(n);
    let m = (reference.lon / dlon).floor()
        + (0.5 + reference.lon.rem_euclid(dlon) / dlon - cpr.lon).floor();
    let mut lon = dlon * (m + cpr.lon);
    if lon >= 180.0 {
        lon -= 360.0;
    }
    if lon < -180.0 {
        lon += 360.0;
    }
    // Odmítnout alias či skok přes 100 km; nejde o primární zdroj polohy.
    let dx = (lon - reference.lon).to_radians() * ((lat + reference.lat) / 2.0).to_radians().cos();
    let dy = (lat - reference.lat).to_radians();
    if 6_371_000.0 * dx.hypot(dy) > 100_000.0 {
        return None;
    }
    Some(Position { lat, lon })
}
#[derive(Default)]
pub struct CprCache {
    entries: BTreeMap<String, (Option<Cpr>, Option<Cpr>, f64)>,
}
impl CprCache {
    /// Vrátí position pouze po přijetí správného even/odd páru.
    pub fn observe(&mut self, key: &str, cpr: Cpr) -> Option<Position> {
        self.observe_with_reference(key, cpr, None)
    }
    pub fn observe_with_reference(
        &mut self,
        key: &str,
        cpr: Cpr,
        reference: Option<&Position>,
    ) -> Option<Position> {
        if !cpr.at.is_finite() || !(0.0..1.0).contains(&cpr.lat) || !(0.0..1.0).contains(&cpr.lon) {
            return None;
        }
        self.entries.retain(|_, (_, _, t)| cpr.at - *t <= 20.0);
        if !self.entries.contains_key(key) && self.entries.len() >= 5000 {
            self.entries.pop_first();
        }
        let pair = self
            .entries
            .entry(key.to_owned())
            .or_insert((None, None, cpr.at));
        if cpr.at < pair.2 - 10.0 {
            return None;
        }
        pair.2 = pair.2.max(cpr.at);
        let target = if cpr.odd { &mut pair.1 } else { &mut pair.0 };
        if target.as_ref().is_some_and(|old| old.at > cpr.at) {
            return None;
        }
        *target = Some(cpr.clone());
        match (&pair.0, &pair.1) {
            (Some(even), Some(odd)) => global_airborne(even, odd),
            _ => None,
        }
        .or_else(|| reference.and_then(|r| local_airborne(&cpr, r)))
    }
}
pub fn observation(decoded: Decoded, position: Option<Position>, received_at: f64) -> Observation {
    Observation {
        receiver_id: "rx-primary".into(),
        transport: "beast_tcp".into(),
        aircraft_key: decoded.key,
        received_at_utc: received_at,
        message_at_utc: received_at,
        time_basis: if position.is_some() {
            "server_arrival"
        } else {
            "unknown"
        }
        .into(),
        position_origin: "adsb".into(),
        position_at_utc: position.as_ref().map(|_| received_at),
        position,
        callsign: decoded.callsign,
        squawk: None,
        telemetry: decoded.telemetry,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::hex_decode;
    #[test]
    fn golden_global_cpr_and_crc_negative() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let mut cache = CprCache::default();
        let mut received = None;
        for (i, case) in f["valid"].as_array().unwrap().iter().take(2).enumerate() {
            let frame = hex_decode(case["modeS"].as_str().unwrap()).unwrap();
            let decoded = decode_df17(&frame, i as f64).unwrap();
            assert_eq!(decoded.key, "icao:abc123");
            received = cache.observe(&decoded.key, decoded.cpr.unwrap());
            if i == 0 {
                assert!(received.is_none());
            }
        }
        let actual = received.unwrap();
        let expected = &f["cprPair"]["decodeReference"];
        assert!(
            (actual.lat - expected["lat"].as_f64().unwrap()).abs() < 1e-4,
            "lat {}",
            actual.lat
        );
        assert!(
            (actual.lon - expected["lon"].as_f64().unwrap()).abs() < 1e-4,
            "lon {}",
            actual.lon
        );
        let invalid = hex_decode(f["invalid"][3]["hex"].as_str().unwrap()).unwrap();
        assert!(decode_df17(&invalid[9..], 2.0).is_none());
    }
    #[test]
    fn altitude_and_trusted_local_cpr() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let frame = hex_decode(cases["valid"][0]["modeS"].as_str().unwrap()).unwrap();
        let decoded = decode_df17(&frame, 1.0).unwrap();
        assert!(decoded.telemetry.altitude_baro_ft.is_some());
        let cpr = decoded.cpr.unwrap();
        let reference = Position {
            lat: 49.9,
            lon: 15.3,
        };
        let position = local_airborne(&cpr, &reference).unwrap();
        assert!((position.lat - 49.9).abs() < 0.01);
        assert!((position.lon - 15.3).abs() < 0.01);
        assert!(
            local_airborne(
                &cpr,
                &Position {
                    lat: f64::NAN,
                    lon: 15.3
                }
            )
            .is_none()
        );
        let mut cache = CprCache::default();
        assert!(cache.observe("icao:abc123", cpr.clone()).is_none());
        assert!(
            cache
                .observe_with_reference("icao:abc123", cpr, Some(&reference))
                .is_some()
        );
    }
    #[test]
    fn subtype_19_velocity_and_bad_crc() {
        // Syntetický DF17 type 19 / subtype 1: rychlost ~100 kt na východ.
        let mut frame = [0u8; 14];
        frame[..4].copy_from_slice(&[0x8d, 0xab, 0xc1, 0x23]);
        let mut me = 0u64;
        let mut put = |first: u32, width: u32, value: u64| {
            me |= value << (57 - first - width);
        };
        put(1, 5, 19);
        put(6, 3, 1);
        put(15, 10, 101);
        put(26, 10, 1);
        put(38, 9, 5);
        for (i, b) in frame[4..11].iter_mut().enumerate() {
            *b = (me >> (8 * (6 - i))) as u8;
        }
        let parity = crc24(&frame[..11]);
        frame[11..].copy_from_slice(&[(parity >> 16) as u8, (parity >> 8) as u8, parity as u8]);
        let d = decode_df17(&frame, 42.0).unwrap();
        assert_eq!(d.telemetry.ground_speed_kt, Some(100.0));
        assert_eq!(d.telemetry.track_degrees, Some(90.0));
        assert_eq!(d.telemetry.vertical_rate_fpm, Some(256));
        frame[13] ^= 1;
        assert!(decode_df17(&frame, 42.0).is_none());
    }
    #[test]
    fn stale_pair_and_wrong_df_no_position() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let a = decode_df17(
            &hex_decode(f["valid"][0]["modeS"].as_str().unwrap()).unwrap(),
            0.0,
        )
        .unwrap();
        let b = decode_df17(
            &hex_decode(f["valid"][1]["modeS"].as_str().unwrap()).unwrap(),
            20.0,
        )
        .unwrap();
        assert!(global_airborne(a.cpr.as_ref().unwrap(), b.cpr.as_ref().unwrap()).is_none());
        assert!(decode_df17(&[0; 7], 0.0).is_none());
        assert!(decode_df17(&[0; 14], 0.0).is_none());
        assert!(observation(a, None, 0.0).position.is_none());
    }
}
