use super::{
    ChannelModeMsg, ChannelVoiceMsg, SystemCommonMsg, SystemExclusiveMsg, SystemRealTimeMsg,
};
use num_derive::FromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiMsg {
    ChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    RunningChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    ChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    RunningChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    SystemCommon {
        msg: SystemCommonMsg,
    },
    SystemRealTime {
        msg: SystemRealTimeMsg,
    },
    SystemExclusive {
        msg: SystemExclusiveMsg,
    },
}

impl MidiMsg {
    pub fn to_midi(self) -> Vec<u8> {
        self.into()
    }

    pub fn from_midi(_m: &[u8]) -> Result<Self, &str> {
        Err("TODO: not implemented")
    }
}

impl From<MidiMsg> for Vec<u8> {
    fn from(m: MidiMsg) -> Vec<u8> {
        match m {
            MidiMsg::ChannelVoice { channel, msg } => {
                let mut r = msg.to_midi();
                r[0] += channel as u8;
                r
            }
            MidiMsg::RunningChannelVoice { msg, .. } => msg.to_midi_running(),
            MidiMsg::ChannelMode { channel, msg } => {
                let mut r = msg.to_midi();
                r[0] += channel as u8;
                r
            }
            MidiMsg::RunningChannelMode { msg, .. } => msg.to_midi_running(),
            MidiMsg::SystemCommon { msg } => msg.to_midi(),
            MidiMsg::SystemRealTime { msg } => msg.to_midi(),
            MidiMsg::SystemExclusive { msg } => msg.to_midi(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
}
