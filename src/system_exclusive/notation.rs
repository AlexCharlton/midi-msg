use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::parse_error::*;
use crate::util::*;
use crate::system_exclusive::util::*;

/// Indicates that the next MIDI clock message is the first clock of a new measure. Which bar
/// is optionally indicated by this message.
/// Used by [`UniversalRealTimeMsg::BarMarker`](crate::UniversalRealTimeMsg::BarMarker).
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BarMarker {
    /// "Actually, we're not running right now, so there is no bar." Don't know why this is used.
    NotRunning,
    /// The bar is a count-in and are thus negative numbers from 8191-0.
    CountIn(u16), // ?
    /// A regular bar numbered 1-8191.
    Number(u16),
    /// Next clock message will be a new bar, but it's not known what its number is.
    RunningUnknown,
}

impl BarMarker {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match *self {
            Self::NotRunning => {
                // Most negative number
                v.push(0x00);
                v.push(0x40);
            }
            Self::CountIn(x) => {
                push_i14(-(x.min(8191) as i16), v);
            }
            Self::Number(x) => {
                push_i14(x.min(8191) as i16, v);
            }
            Self::RunningUnknown => {
                // Most positive number
                v.push(0x7F);
                v.push(0x3F);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// Used to communicate a new time signature to the receiver.
/// Used by [`UniversalRealTimeMsg`](crate::UniversalRealTimeMsg).
#[derive(Debug, Clone, PartialEq)]
pub struct TimeSignature {
    /// The base time signature.
    pub signature: Signature,
    /// How many MIDI clock events per metronome click.
    /// 24 indicates one click per quarter note (unless specified otherwise by `thirty_second_notes_in_midi_quarter_note`)
    pub midi_clocks_in_metronome_click: u8,
    /// Number of notated 32nd notes in a MIDI quarter note.
    /// 8 is the normal value (e.g. a midi quarter note is a quarter note)
    pub thirty_second_notes_in_midi_quarter_note: u8,
    /// At most 61 (!) additional times signatures for compound time definitions
    pub compound: Vec<Signature>,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            signature: Default::default(),
            midi_clocks_in_metronome_click: 24, // one click per quarter note
            thirty_second_notes_in_midi_quarter_note: 8, // Midi quarter note is a quarter note
            compound: vec![],
        }
    }
}

impl TimeSignature {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push((4 + (self.compound.len() * 2)).min(126) as u8); // Bytes to follow
        self.signature.extend_midi(v);
        v.push(to_u7(self.midi_clocks_in_metronome_click));
        v.push(to_u7(self.thirty_second_notes_in_midi_quarter_note));
        let mut i = 0;
        for s in self.compound.iter() {
            if i >= 61 {
                break;
            }
            s.extend_midi(v);
            i += 1;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// A [time signature](https://en.wikipedia.org/wiki/Time_signature). Used by [`TimeSignature`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Signature {
    /// Number of beats in a bar.
    pub beats: u8,
    /// The note value for each beat.
    pub beat_value: BeatValue,
}

impl Signature {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(to_u7(self.beats));
        v.push(self.beat_value.to_u8());
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            beats: 4,
            beat_value: BeatValue::Quarter,
        }
    }
}

/// The note value of a beat, used by [`Signature`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BeatValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    /// Any other value interpreted as 2^x.
    /// The spec does not limit this value, so the maximum is a crazy 2^127
    Other(u8),
}

impl BeatValue {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Whole => 0,
            Self::Half => 1,
            Self::Quarter => 2,
            Self::Eighth => 3,
            Self::Sixteenth => 4,
            Self::ThirtySecond => 5,
            Self::SixtyFourth => 6,
            Self::Other(x) => to_u7(*x),
        }
    }

    #[allow(dead_code)]
    fn from_byte(_m: u8) -> Self {
        // TODO
        Self::Quarter
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

    #[test]
    fn serialize_bar_marker() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::BarMarker(BarMarker::NotRunning),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 01, 0x00, 0x40, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::BarMarker(BarMarker::CountIn(1)),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 01, 0x7f, 0x7f, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::BarMarker(BarMarker::Number(1)),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 01, 0x01, 0x00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::BarMarker(BarMarker::RunningUnknown),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 01, 0x7f, 0x3f, 0xF7]
        );
    }

    #[test]
    fn serialize_time_signature() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeSignatureDelayed(TimeSignature::default()),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 0x42, 4, 4, 2, 24, 8, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeSignature(TimeSignature {
                        compound: vec! {Signature {
                            beats: 3,
                            beat_value: BeatValue::Eighth,
                        }},
                        ..Default::default()
                    }),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 03, 0x02, 6, 4, 2, 24, 8, 3, 3, 0xF7]
        );
    }
}
