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
                r.extend(data.iter().map(|d| to_u7(*d)).collect::<Vec<u8>>());
                r.push(0xF7);
                r
            }
            SystemExclusiveMsg::NonCommercial { data } => {
                let mut r: Vec<u8> = vec![0xF0, 0x7D];
                r.extend(data.iter().map(|d| to_u7(*d)).collect::<Vec<u8>>());
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

impl From<u8> for SysExID {
    fn from(a: u8) -> Self {
        Self(a, None)
    }
}

impl From<(u8, u8)> for SysExID {
    fn from((a, b): (u8, u8)) -> Self {
        Self(a, Some(b))
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
    MachineControlCommand(MachineControlCommandMsg),
    MachineControlResponse(MachineControlResponseMsg),
    TuningNoteChange(TuningNoteChange),
}

impl UniversalRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        self.into()
    }
}

impl From<&UniversalRealTimeMsg> for Vec<u8> {
    fn from(m: &UniversalRealTimeMsg) -> Vec<u8> {
        match m {
            UniversalRealTimeMsg::TimeCodeFull(code) => {
                let [frame, seconds, minutes, codehour] = code.to_bytes();
                vec![01, 01, codehour, minutes, seconds, frame]
            }
            UniversalRealTimeMsg::TimeCodeUserBits(user_bits) => {
                let [ub1, ub2, ub3, ub4, ub5, ub6, ub7, ub8, ub9] = user_bits.to_nibbles();
                vec![01, 02, ub1, ub2, ub3, ub4, ub5, ub6, ub7, ub8, ub9]
            }
            UniversalRealTimeMsg::ShowControl(msg) => {
                let mut r: Vec<u8> = vec![02];
                r.extend(msg.to_midi());
                r
            }
            UniversalRealTimeMsg::BarMarker(marker) => {
                let mut r: Vec<u8> = vec![03, 01];
                r.extend(marker.to_midi());
                r
            }
            UniversalRealTimeMsg::TimeSignature(signature) => {
                let mut r: Vec<u8> = vec![03, 02];
                r.extend(signature.to_midi());
                r
            }
            UniversalRealTimeMsg::TimeSignatureDelayed(signature) => {
                let mut r: Vec<u8> = vec![03, 42];
                r.extend(signature.to_midi());
                r
            }
            UniversalRealTimeMsg::MasterVolume(vol) => {
                let [msb, lsb] = to_u14(*vol);
                vec![04, 01, lsb, msb]
            }
            UniversalRealTimeMsg::MasterBalance(bal) => {
                let [msb, lsb] = to_u14(*bal);
                vec![04, 02, lsb, msb]
            }
            UniversalRealTimeMsg::TimeCodeCueing(msg) => {
                let mut r: Vec<u8> = vec![05];
                r.extend(msg.to_midi());
                r
            }
            UniversalRealTimeMsg::MachineControlCommand(msg) => {
                let mut r: Vec<u8> = vec![06];
                r.extend(msg.to_midi());
                r
            }
            UniversalRealTimeMsg::MachineControlResponse(msg) => {
                let mut r: Vec<u8> = vec![07];
                r.extend(msg.to_midi());
                r
            }
            UniversalRealTimeMsg::TuningNoteChange(note_change) => {
                let mut r: Vec<u8> = vec![08, 02];
                r.extend(note_change.to_midi());
                r
            }
        }
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
    TuningBulkDumpReply(u8, TuningBulkDumpReply),
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
    fn from(m: &UniversalNonRealTimeMsg) -> Vec<u8> {
        match m {
            UniversalNonRealTimeMsg::SampleDump(msg) => {
                let mut r: Vec<u8> = vec![];
                match msg {
                    SampleDumpMsg::Header => r.push(01),
                    SampleDumpMsg::Packet => r.push(02),
                    SampleDumpMsg::Request => r.push(03),
                    SampleDumpMsg::MultipleLoopPoints => {
                        r.push(05);
                        r.push(01);
                    }
                    SampleDumpMsg::LoopPointsRequest => {
                        r.push(05);
                        r.push(02);
                    }
                }
                r.extend(msg.to_midi());
                r
            }
            UniversalNonRealTimeMsg::TimeCode(msg) => {
                let mut r: Vec<u8> = vec![04];
                r.extend(msg.to_midi());
                r
            }
            UniversalNonRealTimeMsg::IdentityRequest => vec![06, 01],
            UniversalNonRealTimeMsg::IdentityReply(identity) => {
                let mut r: Vec<u8> = vec![06, 02];
                r.extend(identity.to_midi());
                r
            }
            UniversalNonRealTimeMsg::FileDump(msg) => {
                let mut r: Vec<u8> = vec![07];
                r.extend(msg.to_midi());
                r
            }
            UniversalNonRealTimeMsg::TuningBulkDumpRequest(program_num) => {
                vec![08, 00, to_u7(*program_num)]
            }
            UniversalNonRealTimeMsg::TuningBulkDumpReply(program_num, tuning) => {
                let mut r: Vec<u8> = vec![08, 01, to_u7(*program_num)];
                r.extend(tuning.to_midi());
                r
            }
            UniversalNonRealTimeMsg::GeneralMidi(on) => vec![09, if *on { 01 } else { 02 }],
            UniversalNonRealTimeMsg::EOF => vec![0x7B, 00],
            UniversalNonRealTimeMsg::Wait => vec![0x7C, 00],
            UniversalNonRealTimeMsg::Cancel => vec![0x7D, 00],
            UniversalNonRealTimeMsg::NAK(packet_num) => vec![0x7E, to_u7(*packet_num)],
            UniversalNonRealTimeMsg::ACK(packet_num) => vec![0x7F, to_u7(*packet_num)],
        }
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
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::Commercial {
                    id: 1.into(),
                    data: vec![0xff, 0x77, 0x00]
                }
            }
            .to_midi(),
            vec![0xF0, 0x01, 0x7F, 0x77, 0x00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::Commercial {
                    id: (1, 3).into(),
                    data: vec![0xff, 0x77, 0x00]
                }
            }
            .to_midi(),
            vec![0xF0, 0x00, 0x01, 0x03, 0x7F, 0x77, 0x00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::NonCommercial {
                    data: vec![0xff, 0x77, 0x00]
                }
            }
            .to_midi(),
            vec![0xF0, 0x7D, 0x7F, 0x77, 0x00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalNonRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalNonRealTimeMsg::EOF
                }
            }
            .to_midi(),
            vec![0xF0, 0x7E, 0x7F, 0x7B, 0x00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::Device(3),
                    msg: UniversalRealTimeMsg::MasterVolume(1000)
                }
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x03, 0x04, 0x01, 0x68, 0x07, 0xF7]
        );
    }
}
