mod controller_destination;
pub use controller_destination::*;
mod file_dump;
pub use file_dump::*;
mod file_reference;
pub use file_reference::*;
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
mod util;

use alloc::vec::Vec;
use alloc::format;
use self::util::checksum;

use super::general_midi::GeneralMidi;
use super::parse_error::*;
use super::time_code::*;
use super::util::*;
use super::ReceiverContext;


/// The bulk of the MIDI spec lives here, in "Universal System Exclusive" messages.
/// Also used for manufacturer-specific messages.
/// Used in [`MidiMsg`](crate::MidiMsg).
#[derive(Debug, Clone, PartialEq)]
pub enum SystemExclusiveMsg {
    /// An arbitrary set of 7-bit "bytes", the meaning of which must be derived from the
    /// message, the definition of which is determined by the given manufacturer.
    Commercial { id: ManufacturerID, data: Vec<u8> },
    /// Similar to `Commercial` but for use in non-commercial situations.
    NonCommercial { data: Vec<u8> },
    /// A diverse range of messages, for real-time applications.
    /// A message is targeted to the given `device`.
    UniversalRealTime {
        device: DeviceID,
        msg: UniversalRealTimeMsg,
    },
    /// A diverse range of messages, for non-real-time applications.
    /// A message is targeted to the given `device`.
    UniversalNonRealTime {
        device: DeviceID,
        msg: UniversalNonRealTimeMsg,
    },
}

impl SystemExclusiveMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
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

    fn sysex_bytes_from_midi(m: &[u8]) -> Result<&[u8], ParseError> {
        if m.first() != Some(&0xF0) {
            return Err(ParseError::Invalid(format!(
                "Undefined System Exclusive message: {:?}",
                m.first()
            )));
        }
        for (i, b) in m[1..].iter().enumerate() {
            if b == &0xF7 {
                return Ok(&m[1..i + 1]);
            }
            if b > &127 {
                return Err(ParseError::ByteOverflow);
            }
        }
        Err(ParseError::NoEndOfSystemExclusiveFlag)
    }

    pub(crate) fn from_midi(
        m: &[u8],
        ctx: &mut ReceiverContext,
    ) -> Result<(Self, usize), ParseError> {
        let m = Self::sysex_bytes_from_midi(m)?;
        match m.get(0) {
            Some(0x7D) => Ok((
                Self::NonCommercial {
                    data: m[1..].to_vec(),
                },
                m.len() + 2,
            )),
            Some(0x7E) => Ok((
                Self::UniversalNonRealTime {
                    device: DeviceID::from_midi(m)?,
                    msg: UniversalNonRealTimeMsg::from_midi(&m[2..])?,
                },
                m.len() + 2,
            )),
            Some(0x7F) => Ok((
                Self::UniversalRealTime {
                    device: DeviceID::from_midi(m)?,
                    msg: UniversalRealTimeMsg::from_midi(&m[2..], ctx)?,
                },
                m.len() + 2,
            )),
            Some(_) => {
                let (id, len) = ManufacturerID::from_midi(m)?;
                Ok((
                    Self::Commercial {
                        id,
                        data: m[len..].to_vec(),
                    },
                    m.len() + 2,
                ))
            }
            None => Err(crate::ParseError::UnexpectedEnd),
        }
    }
}

/// Two 7-bit "bytes", used to identify the manufacturer for [`SystemExclusiveMsg::Commercial`] messages.
/// See [the published list of IDs](https://www.midi.org/specifications-old/item/manufacturer-id-numbers).
///
/// If second byte is None, it is a one-byte ID.
/// The first byte in a one-byte ID may not be greater than 0x7C.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManufacturerID(u8, Option<u8>);

impl ManufacturerID {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        if let Some(second) = self.1 {
            v.push(0x00);
            v.push(to_u7(self.0));
            v.push(to_u7(second));
        } else {
            v.push(self.0.min(0x7C))
        }
    }

    fn from_midi(m: &[u8]) -> Result<(Self, usize), ParseError> {
        let b1 = u7_from_midi(m)?;
        if b1 == 0x00 {
            let b2 = u7_from_midi(&m[1..])?;
            let b3 = u7_from_midi(&m[2..])?;
            Ok((Self(b2, Some(b3)), 3))
        } else {
            Ok((Self(b1, None), 1))
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

/// The device ID being addressed, either a number between 0-126 or `AllCall` (all devices).
/// Used by [`SystemExclusiveMsg::UniversalNonRealTime`] and [`SystemExclusiveMsg::UniversalRealTime`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceID {
    Device(u8),
    AllCall,
}

impl DeviceID {
    fn to_u8(&self) -> u8 {
        match self {
            Self::AllCall => 0x7F,
            Self::Device(x) => to_u7(*x),
        }
    }

    fn from_midi(m: &[u8]) -> Result<Self, ParseError> {
        let b = u7_from_midi(m)?;
        if b == 0x7F {
            Ok(Self::AllCall)
        } else {
            Ok(Self::Device(b))
        }
    }
}

/// A diverse range of messages for real-time applications. Used by [`SystemExclusiveMsg::UniversalRealTime`].
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalRealTimeMsg {
    /// For use when a [`SystemCommonMsg::TimeCodeQuarterFrame`](crate::SystemCommonMsg::TimeCodeQuarterFrame1) is not appropriate:
    /// When rewinding, fast-forwarding, or otherwise locating and cueing, where sending quarter frame
    /// messages continuously would be excessive.
    TimeCodeFull(TimeCode),
    /// Provided for sending SMTPE "user bits", which are application specific.
    TimeCodeUserBits(UserBits),
    /// Used to control equipment for liver performances and installations.
    ShowControl(ShowControlMsg),
    /// Indicates that the next MIDI clock message is the first clock of a new measure.
    BarMarker(BarMarker),
    /// Indicates a change in time signature, effective immediately (or on the next MIDI clock).
    TimeSignature(TimeSignature),
    /// Indicates a change in time signature, effective upon receipt of the next `BarMarker` message.
    TimeSignatureDelayed(TimeSignature),
    /// Change the volume of all sound, from 0 (volume off) to 16383.
    MasterVolume(u16),
    /// Change the balance of all sound, from 0 (hard left) to 8192 (center) to 16383 (hard right).
    MasterBalance(u16),
    /// A value from -8192-8191, used like [`Parameter::FineTuning`](crate::Parameter::FineTuning), but affecting all channels.
    ///
    /// Defined in CA-025.
    MasterFineTuning(i16),
    /// A value from -64-63, used like [`Parameter::CoarseTuning`](crate::Parameter::CoarseTuning), but affecting all channels.
    ///
    /// Defined in CA-025.
    MasterCoarseTuning(i8),
    /// Used to control parameters on a device that affect all sound, e.g. a global reverb.
    GlobalParameterControl(GlobalParameterControl),
    /// Used to define a range of time points.
    TimeCodeCueing(TimeCodeCueingMsg),
    /// Used to control audio recording and production systems.
    MachineControlCommand(MachineControlCommandMsg),
    /// Responses to `MachineControlCommand`.
    MachineControlResponse(MachineControlResponseMsg),
    /// Immediately change the tuning of 1 or more notes.
    TuningNoteChange(TuningNoteChange),
    /// A set of 12 tunings across all octaves targeting a set of channels, to take effect immediately.
    ScaleTuning1Byte(ScaleTuning1Byte),
    /// A set of 12 high-res tunings across all octaves targeting a set of channels, to take effect immediately.
    ScaleTuning2Byte(ScaleTuning2Byte),
    /// Select the destination of a [`ChannelPressure`](crate::ChannelVoiceMsg::ChannelPressure) message.
    ChannelPressureControllerDestination(ControllerDestination),
    /// Select the destination of a [`PolyPressure`](crate::ChannelVoiceMsg::PolyPressure) message.
    PolyphonicKeyPressureControllerDestination(ControllerDestination),
    /// Select the destination of a [`ControlChange`](crate::ChannelVoiceMsg::ControlChange) message.
    ControlChangeControllerDestination(ControlChangeControllerDestination),
    /// Intended to act like Control Change messages, but targeted at an individual key for e.g. changing the release time for individual drum sounds.
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
                let [msb, lsb] = i_to_u14(*t);
                v.push(lsb);
                v.push(msb);
            }
            UniversalRealTimeMsg::MasterCoarseTuning(t) => {
                v.push(04);
                v.push(04);
                v.push(i_to_u7(*t));
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

    fn from_midi(m: &[u8], ctx: &mut ReceiverContext) -> Result<Self, ParseError> {
        if m.len() < 2 {
            return Err(crate::ParseError::UnexpectedEnd);
        }

        match (m[0], m[1]) {
            (01, 01) => {
                if m.len() > 6 {
                    Err(ParseError::Invalid(format!(
                        "Extra bytes after a UniversalRealTimeMsg::TimeCodeFull"
                    )))
                } else {
                    let time_code = TimeCode::from_midi(&m[2..])?;
                    ctx.time_code = time_code;
                    Ok(Self::TimeCodeFull(time_code))
                }
            }
            _ => Err(ParseError::Invalid(format!("TODO: Not implemented"))),
        }
    }
}

/// A diverse range of messages for non-real-time applications. Used by [`SystemExclusiveMsg::UniversalNonRealTime`].
#[derive(Debug, Clone, PartialEq)]
pub enum UniversalNonRealTimeMsg {
    /// Used to transmit sampler data.
    SampleDump(SampleDumpMsg),
    /// Additional ways/features for transmitting sampler data per CA-019.
    ExtendedSampleDump(ExtendedSampleDumpMsg),
    /// Used to define a range of time points per MMA0001.
    TimeCodeCueingSetup(TimeCodeCueingSetupMsg),
    /// Request that the targeted device identify itself.
    IdentityRequest,
    /// The response to an `IdentityRequest`.
    IdentityReply(IdentityReply),
    /// Used to transmit general file data.
    FileDump(FileDumpMsg),
    /// Request a tuning bulk dump for the provided
    /// tuning program number, 0-127, and optional tuning bank number, 0-127
    TuningBulkDumpRequest(u8, Option<u8>),
    /// A "key based" tuning dump, with one tuning for every key.
    KeyBasedTuningDump(KeyBasedTuningDump),
    /// A "1 byte scale" tuning dump, with 12 tunings applied across all octaves.
    ScaleTuningDump1Byte(ScaleTuningDump1Byte),
    /// A "2 byte scale" tuning dump, with 12 tunings applied across all octaves.
    /// Like `ScaleTuningDump1Byte` but higher resolution.
    ScaleTuningDump2Byte(ScaleTuningDump2Byte),
    /// Change the tuning of 1 or more notes for the next sounding of those notes.
    TuningNoteChange(TuningNoteChange),
    /// Similar to `ScaleTuningDump1Byte`, but targets a channel, to take effect the next time a note is sounded.
    ScaleTuning1Byte(ScaleTuning1Byte),
    /// Similar to `ScaleTuningDump2Byte`, but targets a channel, to take effect the next time a note is sounded.
    ScaleTuning2Byte(ScaleTuning2Byte),
    /// Turn on or off General MIDI 1 or 2.
    GeneralMidi(GeneralMidi),
    /// Messages for accessing files on a shared network or filesystem.
    FileReference(FileReferenceMsg),
    /// Used by both `SampleDump` and `FileDump` to indicate all packets have been sent.
    EOF,
    /// Used by both `SampleDump` and `FileDump` from the receiver to request that the sender
    /// does not send any more packets until an `ACK` or `NAK` is sent.
    Wait,
    /// Used to abort an ongoing `SampleDump` or `FileDump`.
    Cancel,
    /// Used by both `SampleDump` and `FileDump` from the receiver to indicate that it did not
    /// receive the last packet correctly.
    NAK(u8),
    /// Used by both `SampleDump` and `FileDump` from the receiver to indicate that it
    /// received the last packet correctly.
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
                    SampleDumpMsg::LoopPointTransmission { .. } => {
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
                    ExtendedSampleDumpMsg::LoopPointTransmission { .. } => v.push(06),
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
            UniversalNonRealTimeMsg::GeneralMidi(gm) => {
                v.push(09);
                v.push(*gm as u8);
            }
            UniversalNonRealTimeMsg::FileReference(msg) => {
                v.push(0x0B);
                match msg {
                    FileReferenceMsg::Open { .. } => v.push(01),
                    FileReferenceMsg::SelectContents { .. } => v.push(02),
                    FileReferenceMsg::OpenSelectContents { .. } => v.push(03),
                    FileReferenceMsg::Close { .. } => v.push(04),
                }
                msg.extend_midi(v);
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

    fn from_midi(m: &[u8]) -> Result<Self, ParseError> {
        if m.len() < 2 {
            return Err(crate::ParseError::UnexpectedEnd);
        }

        match (m[0], m[1]) {
            _ => Err(ParseError::Invalid(format!("TODO: Not implemented"))),
        }
    }
}

/// A response to [`UniversalNonRealTimeMsg::IdentityRequest`], meant to indicate the type of device
/// that this message is sent from.
/// Used by [`UniversalNonRealTimeMsg::IdentityReply`].
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
    use alloc::vec;

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

    #[test]
    fn deserialize_system_exclusive_msg() {
        let mut ctx = ReceiverContext::new();

        test_serialization(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::Commercial {
                    id: 1.into(),
                    data: vec![0x7f, 0x77, 0x00],
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::Commercial {
                    id: (1, 3).into(),
                    data: vec![0x7f, 0x77, 0x00],
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::NonCommercial {
                    data: vec![0x7f, 0x77, 0x00],
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeCodeFull(TimeCode {
                        frames: 29,
                        seconds: 58,
                        minutes: 20,
                        hours: 23,
                        code_type: TimeCodeType::DF30,
                    }),
                },
            },
            &mut ctx,
        );

        assert_eq!(
            ctx.time_code,
            TimeCode {
                frames: 29,
                seconds: 58,
                minutes: 20,
                hours: 23,
                code_type: TimeCodeType::DF30,
            }
        );
    }
}
