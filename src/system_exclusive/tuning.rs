use crate::util::*;
use ascii::AsciiChar;

#[derive(Debug, Clone, PartialEq)]
pub struct TuningNoteChange {
    /// 0-127
    pub tuning_program_num: u8,
    /// At most 127 (MIDI note number, Option<Tuning>) pairs
    /// None represents "No change"
    pub tunings: Vec<(u8, Option<Tuning>)>,
}

impl TuningNoteChange {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u7(self.tuning_program_num, v);
        push_u7(self.tunings.len() as u8, v);
        for (note, tuning) in self.tunings.iter() {
            push_u7(*note, v);
            if let Some(tuning) = tuning {
                tuning.extend_midi(v);
            } else {
                // "No change"
                v.push(0x7F);
                v.push(0x7F);
                v.push(0x7F);
            }
        }
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuningBulkDumpReply {
    /// 0-127
    pub tuning_program_num: u8,
    /// An exactly 16 character name
    name: [AsciiChar; 16],
    /// Should be exactly 128 Tunings with the index of each value = the MIDI note number being tuned.
    /// Excess values will be ignored. If fewer than 128 values are supplied, equal temperament
    /// will be applied to the remaining notes.
    /// None represents "No change"
    pub tunings: Vec<Option<Tuning>>,
}

impl TuningBulkDumpReply {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u7(self.tuning_program_num, v);
        for ch in self.name.iter() {
            v.push(ch.as_byte());
        }
        let mut i = 0;
        loop {
            if i >= 128 {
                break;
            }
            if let Some(tuning) = self.tunings.get(i) {
                if let Some(tuning) = tuning {
                    tuning.extend_midi(v);
                } else {
                    // "No change"
                    v.push(0x7F);
                    v.push(0x7F);
                    v.push(0x7F);
                }
            } else {
                // The equivalent of equal temperament tuning
                push_u7(i as u8, v);
                v.push(0);
                v.push(0);
            }
            i += 1;
        }
        v.push(0); // Checksum <- Will be written over by `SystemExclusiveMsg.extend_midi`
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Tuning {
    /// The semitone corresponding with the same MIDI note number, 0-127
    pub semitone: u8,
    /// Fraction of semitones above the `semitone`, in .0061-cent units.
    /// 0-16383
    pub fraction: u16,
}

impl Tuning {
    pub fn from_freq(freq: f32) -> Self {
        if freq < 8.17358 {
            Self {
                semitone: 0,
                fraction: 0,
            }
        } else if freq > 13289.73 {
            Self {
                semitone: 127,
                fraction: 16383,
            }
        } else {
            let (semitone, c) = freq_to_midi_note_cents(freq);
            Self {
                semitone: semitone as u8,
                fraction: cents_to_u14(c).min(0x3FFE),
            }
        }
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u7(self.semitone, v);
        let [msb, lsb] = to_u14(self.fraction);
        v.push(msb); // For some reason this is the opposite order of everything else???
        v.push(lsb);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use ascii::{AsAsciiStr, AsciiChar};
    use std::convert::TryInto;

    #[test]
    fn serialize_tuning_note_change() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TuningNoteChange(TuningNoteChange {
                        tuning_program_num: 5,
                        tunings: vec![
                            (
                                1,
                                Some(Tuning {
                                    semitone: 1,
                                    fraction: 255,
                                }),
                            ),
                            (
                                0x33,
                                Some(Tuning {
                                    semitone: 0x33,
                                    fraction: 511,
                                }),
                            ),
                            (0x45, None),
                            (0x78, Some(Tuning::from_freq(8372.0630)))
                        ],
                    }),
                },
            }
            .to_midi(),
            &[
                0xF0, 0x7F, 0x7F, 0x08, 0x02, 0x05, 4, // Number of changes
                0x01, 0x01, 0x01, 0x7f, // Tuning 1
                0x33, 0x33, 0x03, 0x7f, // Tuning 2
                0x45, 0x7f, 0x7f, 0x7f, // Tuning 3 (no change)
                0x78, 0x78, 0x00, 0x01, // Tuning 4
                0xF7,
            ]
        );
    }

    #[test]
    fn serialize_tuning_bulk_dump_reply() {
        let packet_msg = MidiMsg::SystemExclusive {
            msg: SystemExclusiveMsg::UniversalNonRealTime {
                device: DeviceID::AllCall,
                msg: UniversalNonRealTimeMsg::TuningBulkDumpReply(TuningBulkDumpReply {
                    tuning_program_num: 5,
                    name: "A tuning program"
                        .as_ascii_str()
                        .unwrap()
                        .as_slice()
                        .try_into()
                        .unwrap(),
                    tunings: vec![Some(Tuning {
                        semitone: 1,
                        fraction: 255,
                    })],
                }),
            },
        }
        .to_midi();

        assert_eq!(packet_msg.len(), 408);
        assert_eq!(
            &packet_msg[0..7],
            &[0xF0, 0x7E, 0x7F, 0x08, 0x01, 0x05, AsciiChar::A.as_byte()]
        );

        assert_eq!(
            &packet_msg[22..31],
            &[
                0x01, 0x01, 0x7f, // Provided tuning
                0x01, 0x00, 0x00, // Default tuning
                0x02, 0x00, 0x00 // Default tuning
            ]
        );
    }
}
