use alloc::vec::Vec;
use alloc::format;
use super::parse_error::*;
use super::time_code::*;
use super::util::*;
use super::ReceiverContext;

/// A fairly limited set of messages, generally for device synchronization.
/// Used in [`MidiMsg`](crate::MidiMsg).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemCommonMsg {
    /// The first of 8 "quarter frame" messages, which are meant to be sent 4 per "frame".
    /// These messages function similarly to [`SystemRealTimeMsg::TimingClock`](crate::SystemRealTimeMsg::TimingClock)
    /// but additionally indicate the specific point in the playback that they refer to, as well
    /// as the frame rate. This means that a full `TimeCode` is send over the course of two frames.
    ///
    /// They are sent in reverse order if time is playing in reverse.
    TimeCodeQuarterFrame1(TimeCode),
    TimeCodeQuarterFrame2(TimeCode),
    TimeCodeQuarterFrame3(TimeCode),
    TimeCodeQuarterFrame4(TimeCode),
    TimeCodeQuarterFrame5(TimeCode),
    TimeCodeQuarterFrame6(TimeCode),
    TimeCodeQuarterFrame7(TimeCode),
    TimeCodeQuarterFrame8(TimeCode),
    /// Indicate the song position, in MIDI beats, where 1 MIDI beat = 6 MIDI clocks. 0-16383
    SongPosition(u16),
    /// Select a song numbered 0-127.
    SongSelect(u8),
    /// Request that the oscillators of an analog synth be tuned.
    TuneRequest,
}

impl SystemCommonMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            SystemCommonMsg::TimeCodeQuarterFrame1(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[0]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame2(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[1]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame3(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[2]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame4(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[3]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame5(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[4]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame6(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[5]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame7(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[6]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame8(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[7]);
            }
            SystemCommonMsg::SongPosition(pos) => {
                v.push(0xF2);
                push_u14(*pos, v);
            }
            SystemCommonMsg::SongSelect(song) => {
                v.push(0xF3);
                v.push(to_u7(*song));
            }
            SystemCommonMsg::TuneRequest => v.push(0xF6),
        }
    }

    pub(crate) fn from_midi(
        m: &[u8],
        ctx: &mut ReceiverContext,
    ) -> Result<(Self, usize), ParseError> {
        match m.first() {
            Some(0xF1) => {
                if let Some(b2) = m.get(1) {
                    if b2 > &127 {
                        Err(ParseError::ByteOverflow)
                    } else {
                        Ok((
                            match ctx.time_code.extend(*b2) {
                                0 => Self::TimeCodeQuarterFrame1(ctx.time_code),
                                1 => Self::TimeCodeQuarterFrame2(ctx.time_code),
                                2 => Self::TimeCodeQuarterFrame3(ctx.time_code),
                                3 => Self::TimeCodeQuarterFrame4(ctx.time_code),
                                4 => Self::TimeCodeQuarterFrame5(ctx.time_code),
                                5 => Self::TimeCodeQuarterFrame6(ctx.time_code),
                                6 => Self::TimeCodeQuarterFrame7(ctx.time_code),
                                7 => Self::TimeCodeQuarterFrame8(ctx.time_code),
                                _ => panic!("Should not be reachable"),
                            },
                            2,
                        ))
                    }
                } else {
                    Err(ParseError::UnexpectedEnd)
                }
            }
            Some(0xF2) => Ok((Self::SongPosition(u14_from_midi(&m[1..])?), 3)),
            Some(0xF3) => Ok((Self::SongSelect(u7_from_midi(&m[1..])?), 2)),
            Some(0xF6) => Ok((Self::TuneRequest, 1)),
            Some(0xF7) => Err(ParseError::Invalid(format!(
                "Unexpected End of System Exclusive flag"
            ))),
            Some(x) => Err(ParseError::Invalid(format!(
                "Undefined System Common message: {}",
                x
            ))),
            _ => panic!("Should not be reachable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    extern crate std;
    use std::vec;

    #[test]
    fn serialize_system_common_msg() {
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TuneRequest
            }
            .to_midi(),
            vec![0xF6]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongSelect(69)
            }
            .to_midi(),
            vec![0xF3, 69]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongPosition(1000)
            }
            .to_midi(),
            vec![0xF2, 0x68, 0x07]
        );

        let frame = TimeCode {
            frames: 40,                    // Should be limited to 29: 0b00011101
            seconds: 58,                   // 0b00111010
            minutes: 20,                   // 0b00010100
            hours: 25,                     // Should be limited to 23: 0b00010111
            code_type: TimeCodeType::DF30, //      0b01000000
        };

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame1(frame)
            }
            .to_midi(),
            vec![0xF1, 0b1101]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame2(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame3(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00100000 + 0b1010]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame4(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00110000 + 0b0011]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame5(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01000000 + 0b0100]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame6(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame7(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01100000 + 0b0111]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame8(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01110000 + 0b0101]
        );
    }

    #[test]
    fn deserialize_system_common_msg() {
        let mut ctx = ReceiverContext::new();

        test_serialization(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TuneRequest,
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongPosition(1000),
            },
            &mut ctx,
        );

        MidiMsg::from_midi_with_context(&[0xF1, 0b1101], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b00010000 + 0b0001], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b00100000 + 0b1010], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b00110000 + 0b0011], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b01000000 + 0b0100], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b01010000 + 0b0001], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b01100000 + 0b0111], &mut ctx)
            .expect("Expected a timecode, got an error");
        MidiMsg::from_midi_with_context(&[0xF1, 0b01110000 + 0b0101], &mut ctx)
            .expect("Expected a timecode, got an error");

        assert_eq!(
            ctx.time_code,
            TimeCode {
                frames: 29,                    // 0b00011101
                seconds: 58,                   // 0b00111010
                minutes: 20,                   // 0b00010100
                hours: 23,                     // 0b00010111
                code_type: TimeCodeType::DF30, // 0b01000000
            }
        );
    }
}
