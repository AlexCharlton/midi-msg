mod controller_destination;
pub use controller_destination::*;
mod file_dump;
pub use file_dump::*;
mod global_parameter;
pub use global_parameter::*;
mod key_based_instrument_control;
pub use key_based_instrument_control::*;
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
        id: ManufacturerID,
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
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(0xF0);
        match self {
            SystemExclusiveMsg::Commercial { id, data } => {
                id.extend_midi(v);
                data.iter().for_each(|d| v.push(to_u7(*d)));
            }
            SystemExclusiveMsg::NonCommercial { data } => {
                v.push(0x7D);
                data.iter().for_each(|d| v.push(to_u7(*d)));
            }
            SystemExclusiveMsg::UniversalRealTime { device, msg } => {
                v.push(0x7F);
                v.push(device.to_u8());
                msg.extend_midi(v);
            }
            SystemExclusiveMsg::UniversalNonRealTime { device, msg } => {
                let p = v.len();
                v.push(0x7E);
                v.push(device.to_u8());
                msg.extend_midi(v);
                if let UniversalNonRealTimeMsg::SampleDump(SampleDumpMsg::Packet { .. }) = msg {
                    let q = v.len();
                    v[q - 1] = checksum(&v[p..q - 1]);
                }
                if let UniversalNonRealTimeMsg::KeyBasedTuningDump(_) = msg {
                    let q = v.len();
                    v[q - 1] = checksum(&v[p..q - 1]);
                }
                if let UniversalNonRealTimeMsg::ScaleTuning1Byte(_) = msg {
                    let q = v.len();
                    v[q - 1] = checksum(&v[p..q - 1]);
                }
                if let UniversalNonRealTimeMsg::ScaleTuning2Byte(_) = msg {
                    let q = v.len();
                    v[q - 1] = checksum(&v[p..q - 1]);
                }
                if let UniversalNonRealTimeMsg::FileDump(FileDumpMsg::Packet { .. }) = msg {
                    let q = v.len();
                    v[q - 1] = checksum(&v[p..q - 1]);
                }
            }
        }
        v.push(0xF7);
    }

    /// Ok results return a MidiMsg and the number of bytes consumed from the input
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

impl From<&SystemExclusiveMsg> for Vec<u8> {
    fn from(m: &SystemExclusiveMsg) -> Vec<u8> {
        m.to_midi()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// If second byte is None, it is a one-byte ID
pub struct ManufacturerID(u8, Option<u8>);

impl ManufacturerID {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        if let Some(second) = self.1 {
            v.push(0x00);
            v.push(to_u7(self.0));
            v.push(to_u7(second));
        } else {
            v.push(to_u7(self.0))
        }
    }
}

impl From<u8> for ManufacturerID {
    fn from(a: u8) -> Self {
        Self(a, None)
    }
}

impl From<(u8, u8)> for ManufacturerID {
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
    fn to_u8(&self) -> u8 {
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
    /// A value from -8192-8191, used like `Parameter::FineTuning`
    /// Defined in CA-025
    MasterFineTuning(i16),
    /// A value from -64-63, used like `Parameter::CoarseTuning`
    /// Defined in CA-025
    MasterCoarseTuning(i8),
    /// Defined in CA-024
    GlobalParameterControl(GlobalParameterControl),
    TimeCodeCueing(TimeCodeCueingMsg),
    MachineControlCommand(MachineControlCommandMsg),
    MachineControlResponse(MachineControlResponseMsg),
    TuningNoteChange(TuningNoteChange),
    ScaleTuning1Byte(ScaleTuning1Byte),
    ScaleTuning2Byte(ScaleTuning2Byte),
    ChannelPressureControllerDestination(ControllerDestination),
    PolyphonicKeyPressureControllerDestination(ControllerDestination),
    ControlChangeControllerDestination(ControlChangeControllerDestination),
    KeyBasedInstrumentControl(KeyBasedInstrumentControl),
}

impl UniversalRealTimeMsg {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            UniversalRealTimeMsg::TimeCodeFull(code) => {
                v.push(01);
                v.push(01);
                code.extend_midi(v);
            }
            UniversalRealTimeMsg::TimeCodeUserBits(user_bits) => {
                v.push(01);
                v.push(02);
                let [ub1, ub2, ub3, ub4, ub5, ub6, ub7, ub8, ub9] = user_bits.to_nibbles();
                v.extend_from_slice(&[ub1, ub2, ub3, ub4, ub5, ub6, ub7, ub8, ub9]);
            }
            UniversalRealTimeMsg::ShowControl(msg) => {
                v.push(02);
                msg.extend_midi(v);
            }
            UniversalRealTimeMsg::BarMarker(marker) => {
                v.push(03);
                v.push(01);
                marker.extend_midi(v);
            }
            UniversalRealTimeMsg::TimeSignature(signature) => {
                v.push(03);
                v.push(02);
                signature.extend_midi(v);
            }
            UniversalRealTimeMsg::TimeSignatureDelayed(signature) => {
                v.push(03);
                v.push(0x42);
                signature.extend_midi(v);
            }
            UniversalRealTimeMsg::MasterVolume(vol) => {
                v.push(04);
                v.push(01);
                push_u14(*vol, v);
            }
            UniversalRealTimeMsg::MasterBalance(bal) => {
                v.push(04);
                v.push(02);
                push_u14(*bal, v);
            }
            UniversalRealTimeMsg::MasterFineTuning(t) => {
                v.push(04);
                v.push(03);
                push_i14(*t, v);
            }
            UniversalRealTimeMsg::MasterCoarseTuning(t) => {
                v.push(04);
                v.push(04);
                push_i7(*t, v);
            }
            UniversalRealTimeMsg::GlobalParameterControl(gp) => {
                v.push(04);
                v.push(05);
                gp.extend_midi(v);
            }
            UniversalRealTimeMsg::TimeCodeCueing(msg) => {
                v.push(05);
                msg.extend_midi(v);
            }
            UniversalRealTimeMsg::MachineControlCommand(msg) => {
                v.push(06);
                msg.extend_midi(v);
            }
            UniversalRealTimeMsg::MachineControlResponse(msg) => {
                v.push(07);
                msg.extend_midi(v);
            }
            UniversalRealTimeMsg::TuningNoteChange(note_change) => {
                v.push(08);
                v.push(if note_change.tuning_bank_num.is_some() {
                    07
                } else {
                    02
                });
                if let Some(bank_num) = note_change.tuning_bank_num {
                    v.push(to_u7(bank_num))
                }
                note_change.extend_midi(v);
            }
            UniversalRealTimeMsg::ScaleTuning1Byte(tuning) => {
                v.push(08);
                v.push(08);
                tuning.extend_midi(v);
            }
            UniversalRealTimeMsg::ScaleTuning2Byte(tuning) => {
                v.push(08);
                v.push(09);
                tuning.extend_midi(v);
            }
            UniversalRealTimeMsg::ChannelPressureControllerDestination(d) => {
                v.push(09);
                v.push(01);
                d.extend_midi(v);
            }
            UniversalRealTimeMsg::PolyphonicKeyPressureControllerDestination(d) => {
                v.push(09);
                v.push(02);
                d.extend_midi(v);
            }
            UniversalRealTimeMsg::ControlChangeControllerDestination(d) => {
                v.push(09);
                v.push(03);
                d.extend_midi(v);
            }
            UniversalRealTimeMsg::KeyBasedInstrumentControl(control) => {
                v.push(0x0A);
                v.push(01);
                control.extend_midi(v);
            }
        }
    }

    fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UniversalNonRealTimeMsg {
    SampleDump(SampleDumpMsg),
    ExtendedSampleDump(ExtendedSampleDumpMsg),
    TimeCodeCueingSetup(TimeCodeCueingSetupMsg),
    IdentityRequest,
    IdentityReply(IdentityReply),
    FileDump(FileDumpMsg),
    // Tuning program number, 0-127, and optional tuning bank number, 0-127
    TuningBulkDumpRequest(u8, Option<u8>),
    KeyBasedTuningDump(KeyBasedTuningDump),
    ScaleTuningDump1Byte(ScaleTuningDump1Byte),
    ScaleTuningDump2Byte(ScaleTuningDump2Byte),
    TuningNoteChange(TuningNoteChange),
    ScaleTuning1Byte(ScaleTuning1Byte),
    ScaleTuning2Byte(ScaleTuning2Byte),
    GeneralMidi(bool),
    EOF,
    Wait,
    Cancel,
    NAK(u8),
    ACK(u8),
}

impl UniversalNonRealTimeMsg {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            UniversalNonRealTimeMsg::SampleDump(msg) => {
                match msg {
                    SampleDumpMsg::Header { .. } => v.push(01),
                    SampleDumpMsg::Packet { .. } => v.push(02),
                    SampleDumpMsg::Request { .. } => v.push(03),
                    SampleDumpMsg::MultipleLoopPoints { .. } => {
                        v.push(05);
                        v.push(01);
                    }
                    SampleDumpMsg::LoopPointsRequest { .. } => {
                        v.push(05);
                        v.push(02);
                    }
                }
                msg.extend_midi(v);
            }
            UniversalNonRealTimeMsg::ExtendedSampleDump(msg) => {
                v.push(05);
                match msg {
                    ExtendedSampleDumpMsg::SampleName { .. } => v.push(03),
                    ExtendedSampleDumpMsg::SampleNameRequest { .. } => v.push(04),
                    ExtendedSampleDumpMsg::Header { .. } => v.push(05),
                    ExtendedSampleDumpMsg::MultipleLoopPoints { .. } => v.push(06),
                    ExtendedSampleDumpMsg::LoopPointsRequest { .. } => v.push(07),
                }
                msg.extend_midi(v);
            }
            UniversalNonRealTimeMsg::TimeCodeCueingSetup(msg) => {
                v.push(04);
                msg.extend_midi(v);
            }
            UniversalNonRealTimeMsg::IdentityRequest => {
                v.push(06);
                v.push(01);
            }
            UniversalNonRealTimeMsg::IdentityReply(identity) => {
                v.push(06);
                v.push(02);
                identity.extend_midi(v);
            }
            UniversalNonRealTimeMsg::FileDump(msg) => {
                v.push(07);
                msg.extend_midi(v);
            }
            UniversalNonRealTimeMsg::TuningBulkDumpRequest(program_num, bank_num) => {
                v.push(08);
                v.push(if bank_num.is_some() { 03 } else { 00 });
                if let Some(bank_num) = bank_num {
                    v.push(to_u7(*bank_num))
                }
                v.push(to_u7(*program_num));
            }
            UniversalNonRealTimeMsg::KeyBasedTuningDump(tuning) => {
                v.push(08);
                v.push(if tuning.tuning_bank_num.is_some() {
                    04
                } else {
                    01
                });
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::ScaleTuningDump1Byte(tuning) => {
                v.push(08);
                v.push(05);
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::ScaleTuningDump2Byte(tuning) => {
                v.push(08);
                v.push(06);
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::TuningNoteChange(tuning) => {
                v.push(08);
                v.push(07);
                if let Some(bank_num) = tuning.tuning_bank_num {
                    v.push(to_u7(bank_num))
                } else {
                    v.push(0); // Fallback to Bank 0
                }
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::ScaleTuning1Byte(tuning) => {
                v.push(08);
                v.push(08);
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::ScaleTuning2Byte(tuning) => {
                v.push(08);
                v.push(09);
                tuning.extend_midi(v);
            }
            UniversalNonRealTimeMsg::GeneralMidi(on) => {
                v.push(09);
                v.push(if *on { 01 } else { 02 });
            }
            UniversalNonRealTimeMsg::EOF => {
                v.push(0x7B);
                v.push(00);
            }
            UniversalNonRealTimeMsg::Wait => {
                v.push(0x7C);
                v.push(00);
            }
            UniversalNonRealTimeMsg::Cancel => {
                v.push(0x7D);
                v.push(00);
            }
            UniversalNonRealTimeMsg::NAK(packet_num) => {
                v.push(0x7E);
                v.push(to_u7(*packet_num));
            }
            UniversalNonRealTimeMsg::ACK(packet_num) => {
                v.push(0x7F);
                v.push(to_u7(*packet_num));
            }
        }
    }

    fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IdentityReply {
    pub id: ManufacturerID,
    pub family: u16,
    pub family_member: u16,
    /// Four values, 0-127, sent in order provided
    pub software_revision: (u8, u8, u8, u8),
}

impl IdentityReply {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        self.id.extend_midi(v);
        push_u14(self.family, v);
        push_u14(self.family_member, v);
        v.push(to_u7(self.software_revision.0));
        v.push(to_u7(self.software_revision.1));
        v.push(to_u7(self.software_revision.2));
        v.push(to_u7(self.software_revision.3));
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
