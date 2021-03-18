use crate::parse_error::*;
use crate::time_code::*;

/// A MIDI Machine Control Command.
/// Used by [`UniversalRealTimeMsg::MachineControlCommand`](crate::UniversalRealTimeMsg::MachineControlCommand).
///
/// Only partially implemented. The `Unimplemented` value can be used to
/// represent commands not supported here.
///
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
#[derive(Debug, Clone, PartialEq)]
pub enum MachineControlCommandMsg {
    Stop,
    Play,
    DeferredPlay,
    FastForward,
    Rewind,
    RecordStrobe,
    RecordExit,
    RecordPause,
    Pause,
    Eject,
    Chase,
    CommandErrorReset,
    MMCReset,
    // Write(), TODO
    /// Only `InformationField::GPO-GP7` are valid
    LocateInformationField(InformationField),
    LocateTarget(StandardTimeCode),
    // Move(InformationField, InformationField), TODO
    // Etc... TODO
    Wait,
    Resume,
    /// Used to represent all unimplemented MCC messages.
    /// Is inherently not guaranteed to be a valid message.
    Unimplemented(Vec<u8>),
}

impl MachineControlCommandMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Stop => v.push(0x01),
            Self::Play => v.push(0x02),
            Self::DeferredPlay => v.push(0x03),
            Self::FastForward => v.push(0x04),
            Self::Rewind => v.push(0x05),
            Self::RecordStrobe => v.push(0x06),
            Self::RecordExit => v.push(0x07),
            Self::RecordPause => v.push(0x08),
            Self::Pause => v.push(0x09),
            Self::Eject => v.push(0x0A),
            Self::Chase => v.push(0x0B),
            Self::CommandErrorReset => v.push(0x0C),
            Self::MMCReset => v.push(0x0D),
            Self::LocateInformationField(f) => {
                v.push(0x44);
                v.push(2); // Byte count
                v.push(0); // Sub command
                v.push(*f as u8);
            }
            Self::LocateTarget(stc) => {
                v.push(0x44);
                v.push(6); // Byte count
                v.push(1); // Sub command
                stc.extend_midi(v);
            }
            Self::Wait => v.push(0x01),
            Self::Resume => v.push(0x01),
            Self::Unimplemented(d) => v.extend_from_slice(d),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// A MIDI Machine Control Information Field, which functions something like an address
///
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InformationField {
    SelectedTimeCode = 0x01,
    SelectedMasterCode = 0x02,
    RequestedOffset = 0x03,
    ActualOffset = 0x04,
    LockDeviation = 0x05,
    GeneratorTimeCode = 0x06,
    MidiTimeCodeInput = 0x07,
    GP0 = 0x08,
    GP1 = 0x09,
    GP2 = 0x0A,
    GP3 = 0x0B,
    GP4 = 0x0C,
    GP5 = 0x0D,
    GP6 = 0x0E,
    GP7 = 0x0F,
    // Etc.
    // TODO
}

/// A MIDI Machine Control Response>
/// Used by [`UniversalRealTimeMsg::MachineControlResponse`](crate::UniversalRealTimeMsg::MachineControlResponse).
///
/// Not implemented. The `Unimplemented` value can be used to represent generic responses.
///
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
#[derive(Debug, Clone, PartialEq)]
pub enum MachineControlResponseMsg {
    /// Used to represent all unimplemented MCR messages.
    /// Is inherently not guaranteed to be a valid message.
    Unimplemented(Vec<u8>),
}

impl MachineControlResponseMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Unimplemented(d) => v.extend_from_slice(d),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

#[doc(hidden)]
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
pub struct StandardSpeed(f32);

impl StandardSpeed {
    #[allow(dead_code)]
    pub(crate) fn extend_midi(&self, _v: &mut Vec<u8>) {
        // TODO
    }
}

#[doc(hidden)]
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
pub struct StandardTrack {
    pub video_active: bool,
    pub time_code_active: bool,
    pub time_code_track_active: bool,
    pub aux_track_a_active: bool,
    pub aux_track_b_active: bool,
    pub track_1_active: bool,
    pub track_2_active: bool,
    pub other_tracks: Vec<bool>,
}

impl StandardTrack {
    #[allow(dead_code)]
    pub(crate) fn extend_midi(&self, _v: &mut Vec<u8>) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn serialize_machine_control_msg() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::MachineControlCommand(
                        MachineControlCommandMsg::Stop
                    ),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 06, 01, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::MachineControlCommand(
                        MachineControlCommandMsg::LocateTarget(StandardTimeCode {
                            seconds: 0x20,
                            code_type: TimeCodeType::FPS24, // Is 0
                            ..Default::default()
                        })
                    ),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7f, // Call call
                06,   // MCC
                0x44, // Locate
                0x06, // Bytes
                01,   // Target
                0, 0, 0x20, 0, 0, // Rest of MTC
                0xF7
            ]
        );
    }
}
