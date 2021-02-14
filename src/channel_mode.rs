use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelModeMsg {
    AllSoundOff,
    ResetAllControllers,
    LocalControl(bool),
    AllNotesOff,
    OmniMode(bool),
    PolyMode(PolyMode),
}

impl ChannelModeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn to_midi_running(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi_running(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(0xB0);
        self.extend_midi_running(v);
    }

    pub fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match self {
            ChannelModeMsg::AllSoundOff => {
                v.push(120);
                v.push(0);
            }
            ChannelModeMsg::ResetAllControllers => {
                v.push(121);
                v.push(0);
            }
            ChannelModeMsg::LocalControl(on) => {
                v.push(122);
                v.push(if *on { 127 } else { 0 });
            }
            ChannelModeMsg::AllNotesOff => {
                v.push(123);
                v.push(0);
            }
            ChannelModeMsg::OmniMode(on) => {
                v.push(if *on { 125 } else { 124 });
                v.push(0);
            }
            ChannelModeMsg::PolyMode(m) => {
                v.push(if *m == PolyMode::Poly { 127 } else { 126 });
                v.push(match *m {
                    PolyMode::Poly => 0,
                    PolyMode::Mono(n) => to_u7(n),
                })
            }
        }
    }

    /// Ok results return a MidiMsg and the number of bytes consumed from the input
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

impl From<&ChannelModeMsg> for Vec<u8> {
    fn from(m: &ChannelModeMsg) -> Vec<u8> {
        m.to_midi()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolyMode {
    Mono(u8),
    Poly,
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn serialize_channel_mode_msg() {
        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![0xB2, 120, 0]
        );

        assert_eq!(
            MidiMsg::RunningChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![120, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::LocalControl(true)
            }
            .to_midi(),
            vec![0xB2, 122, 127]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(true)
            }
            .to_midi(),
            vec![0xB2, 125, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(false)
            }
            .to_midi(),
            vec![0xB2, 124, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Poly)
            }
            .to_midi(),
            vec![0xB2, 127, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Mono(4))
            }
            .to_midi(),
            vec![0xB2, 126, 4]
        );
    }
}
