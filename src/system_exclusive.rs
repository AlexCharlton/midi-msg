use super::util::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemExclusiveMsg {
    Comercial { id: SysExID, data: Vec<u8> },
    NonComercial { data: Vec<u8> },
    UniversalRealTime { msg: UniversalRealTimeMsg },
    UniversalNonRealTime { msg: UniversalNonRealTimeMsg },
}

impl SystemExclusiveMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&SystemExclusiveMsg> for Vec<u8> {
    fn from(_m: &SystemExclusiveMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// If second byte is None, it is a one-byte ID
pub struct SysExID(u8, Option<u8>);

impl SysExID {
    pub fn to_midi(&self) -> Vec<u8> {
        if let Some(second) = self.1 {
            vec![0x00, to_u7(self.0), to_u7(second)]
        } else {
            vec![to_u7(self.0)]
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalRealTimeMsg {
    // TODO
}

impl UniversalRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&UniversalRealTimeMsg> for Vec<u8> {
    fn from(_m: &UniversalRealTimeMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalNonRealTimeMsg {
    // TODO
}

impl UniversalNonRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&UniversalNonRealTimeMsg> for Vec<u8> {
    fn from(_m: &UniversalNonRealTimeMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn serialize_system_exclusive_msg() {
        // TODO
        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch1,
                msg: ChannelVoiceMsg::NoteOn {
                    note: 0x88,
                    velocity: 0xff
                }
            }
            .to_midi(),
            vec![0x90, 0x7f, 127]
        );
    }
}
