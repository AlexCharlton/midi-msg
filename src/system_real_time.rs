use super::parse_error::*;

/// A fairly limited set of messages used for device synchronization.
/// Used in [`MidiMsg`](crate::MidiMsg).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemRealTimeMsg {
    /// Used to synchronize clocks. Sent at a rate of 24 per quarter note.
    TimingClock,
    /// Start at the beginning of the song or sequence.
    Start,
    /// Continue from the current location in the song or sequence.
    Continue,
    /// Stop playback.
    Stop,
    /// Sent every 300ms or less whenever other MIDI data is not sent.
    /// Used to indicate that the given device is still connected.
    ActiveSensing,
    /// Request that all devices are reset to their power-up state.
    SystemReset,
}

impl SystemRealTimeMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::TimingClock => v.push(0xF8),
            Self::Start => v.push(0xFA),
            Self::Continue => v.push(0xFB),
            Self::Stop => v.push(0xFC),
            Self::ActiveSensing => v.push(0xFE),
            Self::SystemReset => v.push(0xFF),
        }
    }

    pub(crate) fn from_midi(m: &[u8]) -> Result<(Self, usize), ParseError> {
        match m.first() {
            Some(0xF8) => Ok((Self::TimingClock, 1)),
            Some(0xFA) => Ok((Self::Start, 1)),
            Some(0xFB) => Ok((Self::Continue, 1)),
            Some(0xFC) => Ok((Self::Stop, 1)),
            Some(0xFE) => Ok((Self::ActiveSensing, 1)),
            Some(0xFF) => Ok((Self::SystemReset, 1)),
            Some(x) => Err(ParseError::Invalid(format!(
                "Undefined System Real Time message: {}",
                x
            ))),
            None => panic!("Should not be reachable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn serialize_system_real_time_msg() {
        assert_eq!(
            MidiMsg::SystemRealTime {
                msg: SystemRealTimeMsg::TimingClock
            }
            .to_midi(),
            vec![0xF8]
        );
    }

    #[test]
    fn deserialize_system_real_time_msg() {
        let mut ctx = ReceiverContext::new();

        test_serialization(
            MidiMsg::SystemRealTime {
                msg: SystemRealTimeMsg::TimingClock,
            },
            &mut ctx,
        );
    }
}
