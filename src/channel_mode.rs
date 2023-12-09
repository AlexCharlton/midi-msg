use alloc::vec::Vec;
use super::parse_error::*;
use crate::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Channel-level messages that should alter the mode of the receiver. Used in [`MidiMsg`](crate::MidiMsg).
pub enum ChannelModeMsg {
    /// Sound playing on the channel should be stopped as soon as possible, per GM2.
    AllSoundOff,
    /// Stop sounding all notes on the channel.
    AllNotesOff,
    /// All controllers should be reset to their default values. GM specifies some of these defaults.
    ResetAllControllers,
    /// An instrument set to `OmniMode(true)` should respond to MIDI messages sent over all channels.
    OmniMode(bool),
    /// Request that the receiver set itself to be monophonic/polyphonic.
    PolyMode(PolyMode),
    /// Used to turn on or off "local control" of a MIDI synthesizer instrument. When the instrument
    /// does not have local control, its controller should only send out MIDI signals while the synthesizer should only respond to remote MIDI messages.
    LocalControl(bool),
}

impl ChannelModeMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(0xB0);
        self.extend_midi_running(v);
    }

    pub(crate) fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match self {
            ChannelModeMsg::AllSoundOff => {
                v.push(120);
                v.push(0);
            }
            ChannelModeMsg::ResetAllControllers => {
                v.push(121);
                v.push(0);
            }
            ChannelModeMsg::LocalControl(on) => {
                v.push(122);
                v.push(if *on { 127 } else { 0 });
            }
            ChannelModeMsg::AllNotesOff => {
                v.push(123);
                v.push(0);
            }
            ChannelModeMsg::OmniMode(on) => {
                v.push(if *on { 125 } else { 124 });
                v.push(0);
            }
            ChannelModeMsg::PolyMode(m) => {
                v.push(if *m == PolyMode::Poly { 127 } else { 126 });
                v.push(match *m {
                    PolyMode::Poly => 0,
                    PolyMode::Mono(n) => n.min(16),
                })
            }
        }
    }

    pub(crate) fn from_midi(m: &[u8]) -> Result<(Self, usize), ParseError> {
        // Skip the status byte since it's already been parsed
        let (msg, len) = ChannelModeMsg::from_midi_running(&m[1..])?;
        Ok((msg, len + 1))
    }

    pub(crate) fn from_midi_running(m: &[u8]) -> Result<(Self, usize), ParseError> {
        if let (Some(b1), Some(b2)) = (m.get(0), m.get(1)) {
            if *b2 > 127 {
                return Err(ParseError::ByteOverflow);
            }
            match (b1, b2) {
                (120, _) => Ok((Self::AllSoundOff, 2)),
                (121, _) => Ok((Self::ResetAllControllers, 2)),
                (122, b2) => Ok((Self::LocalControl(bool_from_u7(*b2)?), 2)),
                (123, _) => Ok((Self::AllNotesOff, 2)),
                (124, _) => Ok((Self::OmniMode(false), 2)),
                (125, _) => Ok((Self::OmniMode(true), 2)),
                (126, b2) => Ok((Self::PolyMode(PolyMode::Mono(u8_from_u7(*b2)?)), 2)),
                (127, _) => Ok((Self::PolyMode(PolyMode::Poly), 2)),
                _ => Err(ParseError::Invalid("This shouldn't be possible")),
            }
        } else {
            Err(ParseError::UnexpectedEnd)
        }
    }
}

/// Used by [`ChannelModeMsg::PolyMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolyMode {
    /// Request that the receiver be monophonic, with the given number M representing the
    /// number of channels that should be dedicated. Since this is sent with a `ChannelModeMsg`
    /// there is already a "base" channel associated with it, and the number of requested channels
    /// should be from this base channel N to N+M. `0` is a special case that directing the receiver
    /// to assign the voices to as many channels as it can receive.
    Mono(u8),
    /// Request the receiver to be polyphonic
    Poly,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

    #[test]
    fn serialize_channel_mode_msg() {
        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![0xB2, 120, 0]
        );

        assert_eq!(
            MidiMsg::RunningChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![120, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::LocalControl(true)
            }
            .to_midi(),
            vec![0xB2, 122, 127]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(true)
            }
            .to_midi(),
            vec![0xB2, 125, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(false)
            }
            .to_midi(),
            vec![0xB2, 124, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Poly)
            }
            .to_midi(),
            vec![0xB2, 127, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Mono(4))
            }
            .to_midi(),
            vec![0xB2, 126, 4]
        );
    }

    #[test]
    fn deserialize_channel_mode_msg() {
        let mut ctx = ReceiverContext::new();

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff,
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::LocalControl(true),
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(true),
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(false),
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Poly),
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Mono(4)),
            },
            &mut ctx,
        );
    }
}
