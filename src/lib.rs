//! This is a library for serializing and deserializing MIDI (byte) streams.
//!
//! ## Creating MIDI byte sequences
//! [`MidiMsg`] is the starting point for all MIDI messages. All `MidiMsg`s
//! can be serialized into a valid MIDI sequence. You can create a `MidiMsg`
//! and turn it into a `Vec<u8>` like so:
//!
//! ```
//! use midi_msg::*;
//!
//! MidiMsg::ChannelVoice {
//!     channel: Channel::Ch1,
//!     msg: ChannelVoiceMsg::NoteOn {
//!         note: 60,
//!         velocity: 127
//!     }
//! }
//! .to_midi();
//! ```
//!
//! ## Deserializing MIDI byte sequences
//! Likewise, byte sequences can be deserialized into `MidiMsg`s with [`MidiMsg::from_midi`]:
//!
//! ```
//! use midi_msg::*;
//!
//! // These are two MIDI 'note on' messages:
//! let midi_bytes: Vec<u8> = vec![
//!    0x93, 0x66, 0x70, // First msg
//!    0x93, 0x55, 0x60, // Second msg
//!];
//!
//! let (msg, len) = MidiMsg::from_midi(&midi_bytes).expect("Not an error");
//!
//! assert_eq!(len, 3);
//! assert_eq!(msg, MidiMsg::ChannelVoice {
//!     channel: Channel::Ch4,
//!     msg: ChannelVoiceMsg::NoteOn {
//!         note: 0x66,
//!         velocity: 0x70
//!     }
//! });
//! ```
//!
//! Where then `len` returned is the number of bytes used when a `MidiMsg` has been deserialized.
//!
//! Similarly, [`MidiMsg::from_midi_with_context`] can be used to track the state associated
//! with a MIDI stream, which is necessary to deserialize certain messages:
//!
//! ```
//! use midi_msg::*;
//!
//! let mut ctx = ReceiverContext::new();
//!
//! // This is a three-byte MIDI 'note on' message followed by a two-byte "running status"
//! // 'note on' message, which inherits its first ("status") byte from the last `ChannelModeMsg`:
//! let midi_bytes: Vec<u8> = vec![
//!    0x93, 0x66, 0x70, // First msg
//!    0x55, 0x60, // Running status msg
//!];
//!
//! let (_msg1, len1) =
//!     MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx).expect("Not an error");
//! let (msg2, len2) =
//!     MidiMsg::from_midi_with_context(&midi_bytes[len1..], &mut ctx).expect("Not an error");
//!
//! assert_eq!(len2, 2);
//! assert_eq!(msg2, MidiMsg::ChannelVoice {
//!     channel: Channel::Ch4,
//!     msg: ChannelVoiceMsg::NoteOn {
//!         note: 0x55,
//!         velocity: 0x60
//!     }
//! });
//! ```
//!
//! The previous message would not have been deserializable without the context:
//!
//! ```should_panic
//! use midi_msg::*;
//!
//! let midi_bytes: Vec<u8> = vec![
//!    0x93, 0x66, 0x70, // First msg
//!    0x55, 0x60, // Running status msg
//!];
//!
//! let (_msg1, len1) = MidiMsg::from_midi(&midi_bytes).expect("Not an error");
//! MidiMsg::from_midi(&midi_bytes[len1..]).unwrap();
//!
//! ```
//!
//! ## Notes
//!
//! See the [readme](https://github.com/AlexCharlton/midi-msg/blob/master/readme.md) for a
//! list of the MIDI Manufacturer Association documents that are referenced throughout these docs.
//!
//! Deserialization of most of `UniversalRealTimeMsg` and `UniversalNonRealTimeMsg` has not
//! yet been implemented.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod util;
pub use util::{
    freq_to_midi_note_cents, freq_to_midi_note_float, midi_note_cents_to_freq,
    midi_note_float_to_freq,
};

mod parse_error;
pub use parse_error::*;
mod context;
pub use context::*;
mod time_code;
pub use time_code::*;

mod channel_voice;
pub use channel_voice::*;
mod channel_mode;
pub use channel_mode::*;
mod general_midi;
pub use general_midi::*;
mod system_common;
pub use system_common::*;
mod system_real_time;
pub use system_real_time::*;
mod system_exclusive;
pub use system_exclusive::*;

mod message;
pub use message::*;

#[allow(unused_imports)]
use crate::alloc::format;
#[cfg(test)]
pub fn test_serialization(msg: MidiMsg, ctx: &mut ReceiverContext) {
    let midi = msg.to_midi();
    let (msg2, len) = MidiMsg::from_midi_with_context(&midi, ctx).expect(&format!(
        "The input message should be serialized into a deserializable stream\nInput: {:?}\nGot: {:#?}",
        &midi,
        &msg
    ));
    assert_eq!(
        midi.len(),
        len,
        "Expected deserializing of {:?} to be of length {} but got {:?} which has length {}",
        &msg,
        midi.len(),
        &msg2,
        len
    );
    assert_eq!(msg, msg2);
}
