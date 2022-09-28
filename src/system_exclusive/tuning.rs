use alloc::vec::Vec;
use alloc::format;
use crate::parse_error::*;
use crate::util::*;

/// Change the tunings of one or more notes, either real-time or not.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg) and [`UniversalRealTimeMsg`](crate::UniversalRealTimeMsg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuningNoteChange {
    /// Which tuning program is targeted, 0-127. See [`Parameter::TuningProgramSelect`](crate::Parameter::TuningProgramSelect).
    pub tuning_program_num: u8,
    /// Which tuning bank is targeted, 0-127. See [`Parameter::TuningBankSelect`](crate::Parameter::TuningBankSelect).
    pub tuning_bank_num: Option<u8>,
    /// At most 127 (MIDI note number, Option<Tuning>) pairs.
    /// A `None` value represents "No change".
    pub tunings: Vec<(u8, Option<Tuning>)>,
}

impl TuningNoteChange {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        // The tuning_bank_num is pushed by the caller if needed
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

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: TuningNoteChange not implemented")))
    }
}

/// Set the tunings of all 128 notes.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBasedTuningDump {
    /// Which tuning program is targeted, 0-127. See [`Parameter::TuningProgramSelect`](crate::Parameter::TuningProgramSelect).
    pub tuning_program_num: u8,
    /// Which tuning bank is targeted, 0-127. See [`Parameter::TuningBankSelect`](crate::Parameter::TuningBankSelect).
    pub tuning_bank_num: Option<u8>,
    /// An exactly 16 character name
    pub name: [u8; 16],
    /// Should be exactly 128 Tunings with the index of each value = the MIDI note number being tuned.
    /// Excess values will be ignored. If fewer than 128 values are supplied, equal temperament
    /// will be applied to the remaining notes.
    /// A `None` value represents "No change".
    pub tunings: Vec<Option<Tuning>>,
}

impl KeyBasedTuningDump {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        if let Some(bank_num) = self.tuning_bank_num {
            v.push(to_u7(bank_num))
        }
        push_u7(self.tuning_program_num, v);
        for ch in self.name.iter() {
            v.push(*ch);
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

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: KeyBasedTuningDump not implemented")))
    }
}

/// Used to represent a tuning by [`TuningNoteChange`] and [`KeyBasedTuningDump`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// Set the tuning of all octaves for a tuning program/bank.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg).
///
/// As defined in MIDI Tuning Updated Specification (CA-020/CA-021/RP-020)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScaleTuningDump1Byte {
    /// Which tuning program is targeted, 0-127. See [`Parameter::TuningProgramSelect`](crate::Parameter::TuningProgramSelect).
    pub tuning_program_num: u8,
    /// Which tuning bank is targeted, 0-127. See [`Parameter::TuningBankSelect`](crate::Parameter::TuningBankSelect).
    pub tuning_bank_num: u8,
    /// An exactly 16 character name.
    pub name: [u8; 16],
    /// 12 semitones of tuning adjustments repeated over all octaves, starting with C
    /// Each value represents that number of cents plus the equal temperament tuning,
    /// from -64 to 63 cents
    pub tuning: [i8; 12],
}

impl ScaleTuningDump1Byte {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u7(self.tuning_bank_num, v);
        push_u7(self.tuning_program_num, v);
        for ch in self.name.iter() {
            v.push(*ch);
        }

        for t in self.tuning.iter() {
            v.push(i_to_u7(*t));
        }

        v.push(0); // Checksum <- Will be written over by `SystemExclusiveMsg.extend_midi`
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ScaleTuningDump1Byte not implemented")))
    }
}

/// Set the high-res tuning of all octaves for a tuning program/bank.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg).
///
/// As defined in MIDI Tuning Updated Specification (CA-020/CA-021/RP-020)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScaleTuningDump2Byte {
    /// Which tuning program is targeted, 0-127. See [`Parameter::TuningProgramSelect`](crate::Parameter::TuningProgramSelect).
    pub tuning_program_num: u8,
    /// Which tuning bank is targeted, 0-127. See [`Parameter::TuningBankSelect`](crate::Parameter::TuningBankSelect).
    pub tuning_bank_num: u8,
    /// An exactly 16 character name.
    pub name: [u8; 16],
    /// 12 semitones of tuning adjustments repeated over all octaves, starting with C
    /// Each value represents that fractional number of cents plus the equal temperament tuning,
    /// from -8192 to 8192 (steps of .012207 cents)
    pub tuning: [i16; 12],
}

impl ScaleTuningDump2Byte {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u7(self.tuning_bank_num, v);
        push_u7(self.tuning_program_num, v);
        for ch in self.name.iter() {
            v.push(*ch);
        }

        for t in self.tuning.iter() {
            let [msb, lsb] = i_to_u14(*t);
            v.push(lsb);
            v.push(msb);
        }

        v.push(0); // Checksum <- Will be written over by `SystemExclusiveMsg.extend_midi`
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ScaleTuningDump2Byte not implemented")))
    }
}

/// Set the tuning of all octaves for a set of channels.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg) and [`UniversalRealTimeMsg`](crate::UniversalRealTimeMsg).
///
/// As defined in MIDI Tuning Updated Specification (CA-020/CA-021/RP-020)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScaleTuning1Byte {
    pub channels: ChannelBitMap,
    /// 12 semitones of tuning adjustments repeated over all octaves, starting with C
    /// Each value represents that number of cents plus the equal temperament tuning,
    /// from -64 to 63 cents
    pub tuning: [i8; 12],
}

impl ScaleTuning1Byte {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        self.channels.extend_midi(v);
        for t in self.tuning.iter() {
            v.push(i_to_u7(*t));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ScaleTuning1Byte not implemented")))
    }
}

/// Set the high-res tuning of all octaves for a set of channels.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg) and [`UniversalRealTimeMsg`](crate::UniversalRealTimeMsg).
///
/// As defined in MIDI Tuning Updated Specification (CA-020/CA-021/RP-020)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ScaleTuning2Byte {
    pub channels: ChannelBitMap,
    /// 12 semitones of tuning adjustments repeated over all octaves, starting with C
    /// Each value represents that fractional number of cents plus the equal temperament tuning,
    /// from -8192 to 8192 (steps of .012207 cents)
    pub tuning: [i16; 12],
}

impl ScaleTuning2Byte {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        self.channels.extend_midi(v);
        for t in self.tuning.iter() {
            let [msb, lsb] = i_to_u14(*t);
            v.push(lsb);
            v.push(msb);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ScaleTuning2Byte not implemented")))
    }
}

/// The set of channels to apply this tuning message to. Used by [`ScaleTuning1Byte`] and [`ScaleTuning2Byte`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct ChannelBitMap {
    pub channel_1: bool,
    pub channel_2: bool,
    pub channel_3: bool,
    pub channel_4: bool,
    pub channel_5: bool,
    pub channel_6: bool,
    pub channel_7: bool,
    pub channel_8: bool,
    pub channel_9: bool,
    pub channel_10: bool,
    pub channel_11: bool,
    pub channel_12: bool,
    pub channel_13: bool,
    pub channel_14: bool,
    pub channel_15: bool,
    pub channel_16: bool,
}

impl ChannelBitMap {
    /// All channels set
    pub fn all() -> Self {
        Self {
            channel_1: true,
            channel_2: true,
            channel_3: true,
            channel_4: true,
            channel_5: true,
            channel_6: true,
            channel_7: true,
            channel_8: true,
            channel_9: true,
            channel_10: true,
            channel_11: true,
            channel_12: true,
            channel_13: true,
            channel_14: true,
            channel_15: true,
            channel_16: true,
        }
    }

    /// No channels set
    pub fn none() -> Self {
        Self::default()
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        let mut byte1: u8 = 0;
        if self.channel_16 {
            byte1 += 1 << 1;
        }
        if self.channel_15 {
            byte1 += 1 << 0;
        }
        v.push(byte1);

        let mut byte2: u8 = 0;
        if self.channel_14 {
            byte2 += 1 << 6;
        }
        if self.channel_13 {
            byte2 += 1 << 5;
        }
        if self.channel_12 {
            byte2 += 1 << 4;
        }
        if self.channel_11 {
            byte2 += 1 << 3;
        }
        if self.channel_10 {
            byte2 += 1 << 2;
        }
        if self.channel_9 {
            byte2 += 1 << 1;
        }
        if self.channel_8 {
            byte2 += 1 << 0;
        }
        v.push(byte2);

        let mut byte3: u8 = 0;
        if self.channel_7 {
            byte3 += 1 << 6;
        }
        if self.channel_6 {
            byte3 += 1 << 5;
        }
        if self.channel_5 {
            byte3 += 1 << 4;
        }
        if self.channel_4 {
            byte3 += 1 << 3;
        }
        if self.channel_3 {
            byte3 += 1 << 2;
        }
        if self.channel_2 {
            byte3 += 1 << 1;
        }
        if self.channel_1 {
            byte3 += 1 << 0;
        }
        v.push(byte3);
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ChannelBitMap not implemented")))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use bstr::B;
    use core::convert::TryInto;
    use alloc::vec;


    #[test]
    fn serialize_tuning_note_change() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TuningNoteChange(TuningNoteChange {
                        tuning_program_num: 5,
                        tuning_bank_num: None,
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
                // 0x78, 0x78, 0x00, 0x01, // Tuning 4, exact
                0x78, 0x78, 0x00, 0x02, // Tuning 4, micromath approximation
                0xF7,
            ]
        );
    }

    #[test]
    fn serialize_tuning_bulk_dump_reply() {
        let packet_msg = MidiMsg::SystemExclusive {
            msg: SystemExclusiveMsg::UniversalNonRealTime {
                device: DeviceID::AllCall,
                msg: UniversalNonRealTimeMsg::KeyBasedTuningDump(KeyBasedTuningDump {
                    tuning_program_num: 5,
                    tuning_bank_num: None,
                    name: B("A tuning program").try_into().unwrap(), // B creates a &[u8], try_into converts it into an array
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
            &[0xF0, 0x7E, 0x7F, 0x08, 0x01, 0x05, b"A"[0]]
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
