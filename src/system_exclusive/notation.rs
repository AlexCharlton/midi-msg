// use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BarMarker {
    //TODO
}

impl BarMarker {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        // TODO
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TimeSignature {
    //TODO
}

impl TimeSignature {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        // TODO
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn serialize_bar_marker() {
        // TODO
        // assert_eq!(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff
        //         }
        //     }
        //     .to_midi(),
        //     vec![0x90, 0x7f, 127]
        // );
    }

    #[test]
    fn serialize_time_signature() {
        // TODO
        // assert_eq!(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff
        //         }
        //     }
        //     .to_midi(),
        //     vec![0x90, 0x7f, 127]
        // );
    }
}
