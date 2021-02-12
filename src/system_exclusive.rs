// use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemExclusiveMsg {
    // TODO
}

impl SystemExclusiveMsg {
    pub fn to_midi(self) -> Vec<u8> {
        self.into()
    }
}

impl From<SystemExclusiveMsg> for Vec<u8> {
    fn from(_m: SystemExclusiveMsg) -> Vec<u8> {
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
