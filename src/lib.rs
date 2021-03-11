//! This is a library for serializing and deserializing MIDI (byte) streams.
//!
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
//! See the [readme](https://github.com/AlexCharlton/midi-msg/blob/master/readme.md) for a
//! list of the MIDI Manufacturer Association documents that are referenced throughout these docs.

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
