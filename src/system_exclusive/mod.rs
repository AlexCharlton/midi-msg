mod file_dump;
pub use file_dump::*;
mod machine_control;
pub use machine_control::*;
mod notation;
pub use notation::*;
pub use sample_dump::*;
mod sample_dump;
pub use sample_dump::*;
mod show_control;
pub use show_control::*;
mod tuning;
pub use tuning::*;

use super::time_code::*;
use super::util::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SystemExclusiveMsg {
    Commercial {
        id: SysExID,
        data: Vec<u8>,
    },
    NonCommercial {
        data: Vec<u8>,
    },
    UniversalRealTime {
        device: DeviceID,
        msg: UniversalRealTimeMsg,
    },
    UniversalNonRealTime {
        device: DeviceID,
        msg: UniversalNonRealTimeMsg,
    },
}

impl SystemExclusiveMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&SystemExclusiveMsg> for Vec<u8> {
    fn from(m: &SystemExclusiveMsg) -> Vec<u8> {
        match m {
            SystemExclusiveMsg::Commercial { id, data } => {
                let mut r: Vec<u8> = vec![0xF0];
                r.extend(id.to_midi());
                r.extend_from_slice(&data);
                r.push(0xF7);
                r
            }
            SystemExclusiveMsg::NonCommercial { data } => {
                let mut r: Vec<u8> = vec![0xF0, 0x7D];
                r.extend_from_slice(&data);
                r.push(0xF7);
                r
            }
            SystemExclusiveMsg::UniversalRealTime { device, msg } => {
                let mut r: Vec<u8> = vec![0xF0, 0x7F, device.to_midi()];
                r.extend(msg.to_midi());
                r.push(0xF7);
                r
            }
            SystemExclusiveMsg::UniversalNonRealTime { device, msg } => {
                let mut r: Vec<u8> = vec![0xF0, 0x7E, device.to_midi()];
                r.extend(msg.to_midi());
                r.push(0xF7);
                r
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// If second byte is None, it is a one-byte ID
pub struct SysExID(u8, Option<u8>);

impl SysExID {
    pub fn to_midi(&self) -> Vec<u8> {
        if let Some(second) = self.1 {
            vec![0x00, to_u7(self.0), to_u7(second)]
        } else {
            vec![to_u7(self.0)]
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceID {
    AllCall,
    Device(u8),
}

impl DeviceID {
    pub fn to_midi(&self) -> u8 {
        match self {
            Self::AllCall => 0x7F,
            Self::Device(x) => to_u7(*x),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalRealTimeMsg {
    TimeCodeFull(TimeCode),
    TimeCodeUserBits(UserBits),
    ShowControl(ShowControlMsg),
    BarMarker(BarMarker),
    TimeSignature(TimeSignature),
    TimeSignatureDelayed(TimeSignature),
    MasterVolume(u16),
    MasterBalance(u16),
    TimeCodeCueing(TimeCodeCueingMsg),
    MachineControl(MachineControlMsg),
    TuningNoteChange(TuningNoteChange),
}

impl UniversalRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&UniversalRealTimeMsg> for Vec<u8> {
    fn from(_m: &UniversalRealTimeMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalNonRealTimeMsg {
    SampleDump(SampleDumpMsg),
    TimeCode(TimeCodeMsg),
    IdentityRequest,
    IdentityReply(IdentityReply),
    FileDump(FileDumpMsg),
    // Tuning program number, 0-127
    TuningBulkDumpRequest(u8),
    TuningBulkDumpReply(TuningBulkDumpReply),
    GeneralMidi(bool),
    EOF,
    Wait,
    Cancel,
    NAK(u8),
    ACK(u8),
}

impl UniversalNonRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&UniversalNonRealTimeMsg> for Vec<u8> {
    fn from(_m: &UniversalNonRealTimeMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IdentityReply {
    // TODO
}

impl IdentityReply {
    pub fn to_midi(&self) -> Vec<u8> {
        //TODO
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn serialize_system_exclusive_msg() {
        // TODO
        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch1,
                msg: ChannelVoiceMsg::NoteOn {
                    note: 0x88,
                    velocity: 0xff
                }
            }
            .to_midi(),
            vec![0x90, 0x7f, 127]
        );
    }
}
