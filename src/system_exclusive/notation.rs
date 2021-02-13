use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BarMarker {
    //TODO
}

impl BarMarker {
    pub fn to_midi(&self) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TimeSignature {
    //TODO
}

impl TimeSignature {
    pub fn to_midi(&self) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
