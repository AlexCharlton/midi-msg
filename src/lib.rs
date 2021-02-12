mod connection;
pub use connection::*;

mod util;

pub mod time_code;
pub use time_code::*;

pub mod message;
pub use message::*;

// pub fn midi_note_on(conn: &mut MidiOutputConnection, note: &MidiNote) {
//     match conn.send(&note.to_note_on().to_midi()) {
//         _ => (),
//     };
// }

// pub fn midi_note_off(conn: &mut MidiOutputConnection, note: &MidiNote) {
//     match conn.send(&note.to_note_off().to_midi()) {
//         _ => (),
//     };
// }

// pub fn play_midi(conn: &mut MidiOutputConnection, notes: &[MidiNote]) {
//     for note in notes.iter() {
//         midi_note_on(conn, note);
//     }
// }

// pub fn stop_midi(conn: &mut MidiOutputConnection, notes: &[MidiNote]) {
//     for note in notes.iter() {
//         midi_note_off(conn, note);
//     }
// }

// pub fn sysex_play(conn: &mut MidiOutputConnection) {
//     match conn.send(&[
//         0xF0, 0x7F, 0x7F, // All call
//         0x06, // MCC
//         0x02, // PLAY
//         0xF7, // End
//     ]) {
//         _ => (),
//     };
// }

// pub fn sysex_stop(conn: &mut MidiOutputConnection) {
//     match conn.send(&[
//         0xF0, 0x7F, 0x7F, // All call
//         0x06, // MCC
//         0x01, // STOP
//         0xF7, // End
//     ]) {
//         _ => (),
//     };
// }

// pub fn sysex_locate(conn: &mut MidiOutputConnection) {
//     match conn.send(&[
//         0xF0, // Sysex
//         0x7F, // Real Time
//         0x7F, // "All call"
//         0x06, // MCC
//         0x44, // LOCATE
//         0x06, // Num bytes
//         0x01, // [TARGET]
//         // TODO
//         0xF7, // End
//     ]) {
//         _ => (),
//     };
// }
