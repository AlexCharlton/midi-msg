use super::{MidiConnection, MidiMsg};
use midir::MidiOutputConnection;

impl MidiConnection for MidiOutputConnection {
    type SendError = ();
    type RecieveError = &'static str;

    fn write(&mut self, msg: &[MidiMsg]) -> Result<(), Self::SendError> {
        let b = if msg.len() > 1 {
            let mut buffer: Vec<u8> = vec![];
            for m in msg.iter() {
                buffer.extend(m.to_midi());
            }
            buffer
        } else if msg.len() == 1 {
            msg[0].to_midi()
        } else {
            return Ok(());
        };

        match self.send(&b) {
            Ok(()) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn read(&mut self, _msg: &[u8]) -> Result<Vec<MidiMsg>, Self::RecieveError> {
        Err("Not implemented")
    }
}
