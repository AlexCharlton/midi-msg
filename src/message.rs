use super::{
    ChannelModeMsg, ChannelVoiceMsg, SystemCommonMsg, SystemExclusiveMsg, SystemRealTimeMsg,
};
use num_derive::FromPrimitive;

/// The primary interface of this library. Used to encode MIDI messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiMsg {
    /// Channel-level messages that act on a voice.
    ChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    /// Like `ChannelVoice`, but with the first "status" byte of the message omitted.
    /// When these "running status" messages are sent, the receiver must treat them
    /// as implicitly referring to the previous "status" received.
    ///
    /// For instance, if a `ChannelVoiceMsg::NoteOn` message is received, and then
    /// the next message does not contain a status byte, it implicitly refers to a
    /// `ChannelVoiceMsg::NoteOn`.
    RunningChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    /// Channel-level messages that should alter the mode of the receiver.
    ChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    /// Like `RunningChannelVoice` but for `ChannelMode`
    RunningChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    /// A fairly limited set of messages, generally for device synchronization.
    SystemCommon { msg: SystemCommonMsg },
    /// Another limited set of messages used for device synchronization.
    SystemRealTime { msg: SystemRealTimeMsg },
    /// The bulk of the MIDI spec lives here, in "Universal System Exclusive" messages.
    /// Also the home of manufacturer-specific messages.
    SystemExclusive { msg: SystemExclusiveMsg },
}

impl MidiMsg {
    /// Turn a `MidiMsg` into a series of bytes.
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    #[doc(hidden)]
    /// Turn a series of bytes into a `MidiMsg`.
    ///
    /// Ok results return a MidiMsg and the number of bytes consumed from the input.
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }

    /// Turn a set of `MidiMsg`s into a series of bytes, with fewer allocations than
    /// repeatedly concatenating the results of `to_midi`.
    pub fn messages_to_midi(msgs: &[Self]) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        for m in msgs.iter() {
            m.extend_midi(&mut r);
        }
        r
    }

    /// Given a `Vec<u8>`, append this `MidiMsg` to it.
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
}

impl From<&MidiMsg> for Vec<u8> {
    fn from(m: &MidiMsg) -> Vec<u8> {
        m.to_midi()
    }
}

/// The MIDI channel, 1-16. Used by [`MidiMsg`] and elsewhere.
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
