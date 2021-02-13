use crate::util::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TuningNoteChange {
    //TODO
}

impl TuningNoteChange {
    pub fn to_midi(&self) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TuningBulkDumpReply {
    //TODO
}

impl TuningBulkDumpReply {
    pub fn to_midi(&self) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_tuning_note_change() {
        // TODO
        // assert_eq!(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff
        //         }
        //     }
        //     .to_midi(),
        //     vec![0x90, 0x7f, 127]
        // );
    }

    #[test]
    fn serialize_tuning_bulk_dump_reply() {
        // TODO
        // assert_eq!(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff
        //         }
        //     }
        //     .to_midi(),
        //     vec![0x90, 0x7f, 127]
        // );
    }
}
