use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SampleDumpMsg {
    //TODO
}

impl SampleDumpMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&SampleDumpMsg> for Vec<u8> {
    fn from(_m: &SampleDumpMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
