use super::{
    ChannelModeMsg, ChannelVoiceMsg, SystemCommonMsg, SystemExclusiveMsg, SystemRealTimeMsg,
};
use num_derive::FromPrimitive;

#[derive(Debug, Clone, PartialEq)]
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
    pub fn messages_to_midi(msgs: &[Self]) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        for m in msgs.iter() {
            m.extend_midi(&mut r);
        }
        r
    }

    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            MidiMsg::ChannelVoice { channel, msg } => {
                let p = v.len();
                msg.extend_midi(v);
                v[p] += *channel as u8;
                match msg {
                    ChannelVoiceMsg::HighResNoteOff { .. }
                    | ChannelVoiceMsg::HighResNoteOn { .. } => {
                        v[p + 3] += *channel as u8;
                    }
                    _ => (),
                }
            }
            MidiMsg::RunningChannelVoice { msg, .. } => msg.extend_midi_running(v),
            MidiMsg::ChannelMode { channel, msg } => {
                let p = v.len();
                msg.extend_midi(v);
                v[p] += *channel as u8;
            }
            MidiMsg::RunningChannelMode { msg, .. } => msg.extend_midi_running(v),
            MidiMsg::SystemCommon { msg } => msg.extend_midi(v),
            MidiMsg::SystemRealTime { msg } => msg.extend_midi(v),
            MidiMsg::SystemExclusive { msg } => msg.extend_midi(v),
        }
    }

    /// Ok results return a MidiMsg and the number of bytes consumed from the input
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

impl From<&MidiMsg> for Vec<u8> {
    fn from(m: &MidiMsg) -> Vec<u8> {
        m.to_midi()
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
