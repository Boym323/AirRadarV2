//! G-02.03: inkrementální a paměťově omezený Beast TCP rámcovač.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub kind: u8,
    pub receiver_ticks: u64,
    pub rssi: u8,
    pub payload: Vec<u8>,
}
#[derive(Default)]
pub struct Decoder {
    mode: Mode,
    kind: u8,
    payload_len: usize,
    decoded: Vec<u8>,
    pub bad_frames: u64,
    pub dropped_frames: u64,
}
#[derive(Default)]
enum Mode {
    #[default]
    Seeking,
    Type,
    Data,
    Escaped,
}

impl Decoder {
    /// Omezeno na 256 výstupních rámců za volání. Jednotlivý nedokončený
    /// rámec nikdy nenaroste nad 21 bajtů.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Frame> {
        let mut out = Vec::new();
        for &byte in chunk {
            match self.mode {
                Mode::Seeking => {
                    if byte == 0x1a {
                        self.mode = Mode::Type;
                    }
                }
                Mode::Type => {
                    self.payload_len = match byte {
                        0x31 => 2,
                        0x32 => 7,
                        0x33 => 14,
                        0x1a => {
                            continue;
                        }
                        _ => {
                            self.bad_frames += 1;
                            self.mode = Mode::Seeking;
                            continue;
                        }
                    };
                    self.kind = byte;
                    self.decoded.clear();
                    self.mode = Mode::Data;
                }
                Mode::Data => {
                    if byte == 0x1a {
                        self.mode = Mode::Escaped;
                    } else {
                        self.append(byte, &mut out);
                    }
                }
                Mode::Escaped => {
                    if byte == 0x1a {
                        self.append(0x1a, &mut out);
                    } else {
                        // Neočekávaný start jiného rámce: přejít na novou hlavičku.
                        self.bad_frames += 1;
                        self.decoded.clear();
                        self.mode = Mode::Seeking;
                        if byte == 0x31 || byte == 0x32 || byte == 0x33 {
                            self.mode = Mode::Data;
                            self.kind = byte;
                            self.payload_len = if byte == 0x31 {
                                2
                            } else if byte == 0x32 {
                                7
                            } else {
                                14
                            };
                        }
                    }
                }
            }
        }
        out
    }
    fn append(&mut self, byte: u8, out: &mut Vec<Frame>) {
        self.decoded.push(byte);
        self.mode = Mode::Data;
        if self.decoded.len() == self.payload_len + 7 {
            let ticks = self.decoded[..6]
                .iter()
                .fold(0_u64, |acc, b| (acc << 8) | u64::from(*b));
            if out.len() < 256 {
                out.push(Frame {
                    kind: self.kind,
                    receiver_ticks: ticks,
                    rssi: self.decoded[6],
                    payload: self.decoded[7..].to_vec(),
                });
            } else {
                self.dropped_frames += 1;
            }
            self.decoded.clear();
            self.mode = Mode::Seeking;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::hex_decode;
    #[test]
    fn all_fragments_and_multiple_frames() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/beast/cases.json"
        ))
        .unwrap();
        let mut data = Vec::new();
        for c in cases["valid"].as_array().unwrap() {
            data.extend(hex_decode(c["hex"].as_str().unwrap()).unwrap());
        }
        for step in [1, 2, 5, 31, 512] {
            let mut parser = Decoder::default();
            let mut frames = Vec::new();
            for part in data.chunks(step) {
                frames.extend(parser.push(part));
            }
            assert_eq!(frames.len(), 4, "fragmentation={step}");
            assert_eq!(frames[0].receiver_ticks, 12_000_000);
            assert_eq!(frames[1].payload.len(), 14);
            assert_eq!(frames[2].payload.len(), 7);
            assert_eq!(frames[3].payload, [0x1a, 0x00]);
        }
    }
    #[test]
    fn corrupt_recovery_and_bounded_output() {
        let data = hex_decode("1a35001a33000000b71b001a1a8dabc12358378144453ae11de076").unwrap();
        let mut decoder = Decoder::default();
        let frames = decoder.push(&data);
        assert_eq!(frames.len(), 1);
        assert!(decoder.bad_frames > 0);
        let mut long = Vec::new();
        for _ in 0..300 {
            long.extend_from_slice(&data[3..]);
        }
        let frames = decoder.push(&long);
        assert_eq!(frames.len(), 256);
        assert!(decoder.dropped_frames > 0);
    }
}
