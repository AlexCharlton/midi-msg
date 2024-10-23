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
    pub(crate) previous_channel_message: Option<MidiMsg>,
    pub(crate) time_code: TimeCode,
    pub(crate) is_smf_sysex: bool,
    pub(crate) parsing_smf: bool,
    /// If true, CC messages will be treated as complex CC messages, with their semantics taken from the Midi spec. Otherwise, they will be treated as simple CC messages - i.e. [`ControlChange::CC`](crate::ControlChange::CC).
    pub complex_cc: bool,
}

impl ReceiverContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interpret CC messages as complex CC messages.
    pub fn complex_cc(mut self) -> Self {
        self.complex_cc = true;
        self
    }

    pub(crate) fn parsing_smf(mut self) -> Self {
        self.parsing_smf = true;
        self
    }
}
