use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MachineControlCommandMsg {
    //TODO
}

impl MachineControlCommandMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&MachineControlCommandMsg> for Vec<u8> {
    fn from(_m: &MachineControlCommandMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MachineControlResponseMsg {
    //TODO
}

impl MachineControlResponseMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&MachineControlResponseMsg> for Vec<u8> {
    fn from(_m: &MachineControlResponseMsg) -> Vec<u8> {
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
