use super::{
    ChannelModeMsg, ChannelVoiceMsg, ParseError, ReceiverContext, SystemCommonMsg,
    SystemExclusiveMsg, SystemRealTimeMsg,
};

/// The primary interface of this library. Used to encode MIDI messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MidiMsg {
    /// Channel-level messages that act on a voice, such as turning notes on and off.
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
    pub fn from_midi(m: &[u8]) -> Result<(Self, usize), ParseError> {
        Self::from_midi_with_context(m, &mut ReceiverContext::default())
    }

    #[doc(hidden)]
    /// Turn a series of bytes into a `MidiMsg`, given a [`ReceiverContext`](crate::ReceiverContext).
    ///
    /// Ok results return a MidiMsg and the number of bytes consumed from the input.
    pub fn from_midi_with_context(
        m: &[u8],
        ctx: &mut ReceiverContext,
    ) -> Result<(Self, usize), ParseError> {
        match m.first() {
            Some(b) => match b >> 4 {
                0x8 | 0x9 | 0xA | 0xC | 0xD | 0xE => {
                    let (msg, len) = ChannelVoiceMsg::from_midi(m)?;
                    let channel = Channel::from_u8(b & 0x0F);
                    let midi_msg = Self::ChannelVoice { channel, msg };
                    ctx.previous_channel_message = Some(midi_msg.clone());
                    Ok((midi_msg, len))
                }
                0xB => {
                    // Could either be a Channel Mode or CC message
                    let channel = Channel::from_u8(b & 0x0F);
                    let (midi_msg, len) = if let Some(b2) = m.get(1) {
                        if b2 >= &120 {
                            let (msg, len) = ChannelModeMsg::from_midi(m)?;
                            (Self::ChannelMode { channel, msg }, len)
                        } else {
                            let (msg, len) = ChannelVoiceMsg::from_midi(m)?;
                            (Self::ChannelVoice { channel, msg }, len)
                        }
                    } else {
                        return Err(ParseError::UnexpectedEnd);
                    };
                    ctx.previous_channel_message = Some(midi_msg.clone());
                    Ok((midi_msg, len))
                }
                0xF => Err(ParseError::Invalid("TODO".to_string())),
                _ => {
                    if let Some(p) = &ctx.previous_channel_message {
                        match p {
                            Self::ChannelVoice {channel, msg} => {
                                let (msg, len) = ChannelVoiceMsg::from_midi_running(m, msg)?;
                                Ok((Self::ChannelVoice { channel: *channel, msg}, len))
                            }
                            Self::ChannelMode {channel, ..} => {
                                let (msg, len) = ChannelModeMsg::from_midi_running(m)?;
                                Ok((Self::ChannelMode { channel: *channel, msg}, len))
                            }
                            _ => Err(ParseError::Invalid(format!("ReceiverContext::previous_channel_message may only be a ChannelMode or ChannelVoice message.")))
                        }
                    } else {
                        Err(ParseError::ContextlessRunningStatus)
                    }
                }
            },
            None => Err(ParseError::UnexpectedEnd),
        }
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
#[derive(Debug, Clone, Copy, PartialEq)]
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

impl Channel {
    pub fn from_u8(x: u8) -> Self {
        match x {
            0 => Self::Ch1,
            1 => Self::Ch2,
            2 => Self::Ch3,
            3 => Self::Ch4,
            4 => Self::Ch5,
            5 => Self::Ch6,
            6 => Self::Ch7,
            7 => Self::Ch8,
            8 => Self::Ch9,
            9 => Self::Ch10,
            10 => Self::Ch11,
            11 => Self::Ch12,
            12 => Self::Ch13,
            13 => Self::Ch14,
            14 => Self::Ch15,
            _ => Self::Ch16,
        }
    }
}
