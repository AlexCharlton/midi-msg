use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MachineControlMsg {
    //TODO
}

impl MachineControlMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&MachineControlMsg> for Vec<u8> {
    fn from(_m: &MachineControlMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_Machine_control_msg() {
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
