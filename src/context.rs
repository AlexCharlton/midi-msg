use super::{MidiMsg, TimeCode};

/// Passed to [`MidiMsg::from_midi_with_context`](crate::MidiMsg::from_midi_with_context) to allow
/// for the capture and use of captured context while reading from a MIDI stream.
///
/// This is used to allow for the formation of fully formed `MidiMsg`s when either a running
/// status is being employed, or when using 14-bit [`ControlChange`](crate::ControlChange) messages.
///
/// It's also used to track the current [`TimeCode`](crate::TimeCode)
/// as sent through [`SystemCommonMsg::TimeCodeQuarterFrame`](crate::SystemCommonMsg::TimeCodeQuarterFrame1)
/// messages, or [`UniversalRealTimeMsg::TimeCodeFull`](crate::UniversalRealTimeMsg::TimeCodeFull)
/// messages.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReceiverContext {
    pub previous_channel_message: Option<MidiMsg>,
    pub time_code: TimeCode,
    pub is_smf_sysex: bool,
}

impl ReceiverContext {
    pub fn new() -> Self {
        Self::default()
    }
}
