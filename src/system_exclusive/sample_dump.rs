// use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SampleDumpMsg {
    Header,
    Packet,
    Request,
    MultipleLoopPoints,
    LoopPointsRequest,
}

impl SampleDumpMsg {
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
    fn serialize_sample_dump_msg() {
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
