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
//! ## Midi files
//! We can work with Standard Midi Files (SMF) in much the same way. The [`MidiFile`] struct represents this data type and it can be serialized into a `Vec<u8>` and deserialized from a `Vec<u8>`. It holds a header and a list of tracks. Regular [`Track::Midi`] tracks contains a list of [`MidiMsg`] events along with the "delta time" that separates subsequent ones. The definition of the delta time is given by the `division` field in the [`Header`].
//!
//! Convenience functions are provided for constructing a `MidiFile` based on a series of events and absolute beat or frame timings. For example, the following creates a `MidiFile` with a single track containing a single note.
//!
//! ```
//! # #[cfg(feature = "file")]
//! # fn main() {
//! use midi_msg::*;
//!
//! let mut file = MidiFile::default();
//! // Add a track, updating the header with the number of tracks:
//! file.add_track(Track::default());
//! // Add a note on message at beat 0:
//! file.extend_track(0, MidiMsg::ChannelVoice {
//!     channel: Channel::Ch1,
//!     msg: ChannelVoiceMsg::NoteOn {
//!         note: 60,
//!         velocity: 127
//!     }
//! }, 0.0);
//! // Add a note off message at beat 1:
//! file.extend_track(0, MidiMsg::ChannelVoice {
//!     channel: Channel::Ch1,
//!     msg: ChannelVoiceMsg::NoteOff {
//!         note: 60,
//!         velocity: 0
//!     }
//! }, 1.0);
//! // Add an end of track message at beat 4,
//! // which is the only required (by the spec) message in a track:
//! file.extend_track(0, MidiMsg::Meta { msg: Meta::EndOfTrack }, 4.0);
//!
//! // Now we can serialize the track to a Vec<u8>:
//! let midi_bytes = file.to_midi();
//! // And we can deserialize it back to a MidiFile:
//! let file2 = MidiFile::from_midi(&midi_bytes).unwrap();
//! assert_eq!(file, file2);
//! # }
//!
//! # #[cfg(not(feature = "file"))]
//! # fn main() {}
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
#[cfg(feature = "sysex")]
mod system_exclusive;
#[cfg(feature = "sysex")]
pub use system_exclusive::*;
#[cfg(feature = "file")]
mod file;
#[cfg(feature = "file")]
pub use file::*;

mod message;
pub use message::*;

// A helper used in tests
#[cfg(test)]
pub fn test_serialization(msg: MidiMsg, ctx: &mut ReceiverContext) {
    #[cfg(not(feature = "std"))]
    use crate::alloc::format;

    let midi = msg.to_midi();
    let (msg2, len) = MidiMsg::from_midi_with_context(&midi, ctx).unwrap_or_else(|_| panic!("The input message should be serialized into a deserializable stream\nInput: {:?}\nGot: {:#?}",
        &midi,
        &msg));
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
