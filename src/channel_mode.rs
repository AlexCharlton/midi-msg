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
        let m = self.to_midi_running();
        vec![0xB0, m[0], m[1]]
    }

    pub fn to_midi_running(&self) -> Vec<u8> {
        match self {
            ChannelModeMsg::AllSoundOff => vec![120, 0],
            ChannelModeMsg::ResetAllControllers => vec![121, 0],
            ChannelModeMsg::LocalControl(on) => vec![122, if *on { 127 } else { 0 }],
            ChannelModeMsg::AllNotesOff => vec![123, 0],
            ChannelModeMsg::OmniMode(on) => vec![if *on { 125 } else { 124 }, 0],
            ChannelModeMsg::PolyMode(m) => vec![
                if *m == PolyMode::Poly { 127 } else { 126 },
                match *m {
                    PolyMode::Poly => 0,
                    PolyMode::Mono(n) => to_u7(n),
                },
            ],
        }
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
