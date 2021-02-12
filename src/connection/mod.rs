#[cfg(feature = "midir_connection")]
mod midir;
pub use self::midir::*;

use super::MidiMsg;

pub trait MidiConnection {
    type SendError;
    type RecieveError;

    fn write(&mut self, msg: &[MidiMsg]) -> Result<(), Self::SendError>;
    fn read(&mut self, msg: &[u8]) -> Result<Vec<MidiMsg>, Self::RecieveError>;

    fn sysex_play(&mut self) {
        // TODO self.write
    }
}
