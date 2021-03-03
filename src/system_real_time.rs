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
            SystemRealTimeMsg::TimingClock => v.push(0xF8),
            SystemRealTimeMsg::Start => v.push(0xFA),
            SystemRealTimeMsg::Continue => v.push(0xFB),
            SystemRealTimeMsg::Stop => v.push(0xFC),
            SystemRealTimeMsg::ActiveSensing => v.push(0xFE),
            SystemRealTimeMsg::SystemReset => v.push(0xFF),
        }
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

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
}
