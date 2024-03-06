use crate::parse_error::*;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A MIDI Show Control command.
/// Used by [`UniversalRealTimeMsg::ShowControl`](crate::UniversalRealTimeMsg::ShowControl).
///
/// Unimplemented, though the `Unimplemented` value can be used to
/// represent the commands not supported here.
///
/// As defined in MIDI Show Control 1.1.1 (RP002/RP014)
pub enum ShowControlMsg {
    /// Used to represent all unimplemented MSC messages.
    /// Is inherently not guaranteed to be a valid message.
    Unimplemented(Vec<u8>),
}

impl ShowControlMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Unimplemented(d) => v.extend_from_slice(d),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::NotImplemented("ShowControlMsg"))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn serialize_show_control_msg() {
        // TODO
    }
}
