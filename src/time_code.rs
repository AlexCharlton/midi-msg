use super::util::*;
use crate::MidiMsg;
use ascii::AsciiString;

/// Used to synchronize device positions, by [`SystemCommonMsg::TimeCodeQuarterFrameX`](crate::SystemCommonMsg::TimeCodeQuarterFrame1)
/// as well as [`UniversalRealTimeMsg::TimeCodeFull`](crate::UniversalRealTimeMsg::TimeCodeFull).
///
/// Based on [the SMTPE time code standard](https://en.wikipedia.org/wiki/SMPTE_timecode).
///
/// As defined in the MIDI Time Code spec (MMA0001 / RP004 / RP008)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeCode {
    /// The position in frames, 0-29
    pub frames: u8,
    /// The position in seconds, 0-59
    pub seconds: u8,
    /// The position in minutes, 0-59
    pub minutes: u8,
    /// The position in hours, 0-23
    pub hours: u8,
    pub code_type: TimeCodeType,
}

impl TimeCode {
    /// Return the four byte representation of the frame: [frame, seconds, minutes, timecode + hours]
    pub fn to_bytes(self) -> [u8; 4] {
        [
            self.frames.min(29),
            self.seconds.min(59),
            self.minutes.min(59),
            self.hours.min(23) + ((self.code_type as u8) << 5),
        ]
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        let [frame, seconds, minutes, codehour] = self.to_bytes();
        v.extend_from_slice(&[codehour, minutes, seconds, frame]);
    }

    /// Return an 8 byte, Quarter Frame representation of the Frame
    pub fn to_nibbles(self) -> [u8; 8] {
        let [frame, seconds, minutes, codehour] = self.to_bytes();

        let [frame_msb, frame_lsb] = to_nibble(frame);
        let [seconds_msb, seconds_lsb] = to_nibble(seconds);
        let [minutes_msb, minutes_lsb] = to_nibble(minutes);
        let [codehour_msb, codehour_lsb] = to_nibble(codehour);
        [
            (0 << 4) + frame_lsb,
            (1 << 4) + frame_msb,
            (2 << 4) + seconds_lsb,
            (3 << 4) + seconds_msb,
            (4 << 4) + minutes_lsb,
            (5 << 4) + minutes_msb,
            (6 << 4) + codehour_lsb,
            (7 << 4) + codehour_msb,
        ]
    }

    // Returns the quarter frame number
    pub(crate) fn extend(&mut self, nibble: u8) -> u8 {
        let frame_number = nibble >> 4;
        let nibble = nibble & 0b00001111;

        match frame_number {
            0 => self.frames = (self.frames & 0b11110000) + nibble,
            1 => self.frames = (self.frames & 0b00001111) + (nibble << 4),
            2 => self.seconds = (self.seconds & 0b11110000) + nibble,
            3 => self.seconds = (self.seconds & 0b00001111) + (nibble << 4),
            4 => self.minutes = (self.minutes & 0b11110000) + nibble,
            5 => self.minutes = (self.minutes & 0b00001111) + (nibble << 4),
            6 => self.hours = (self.hours & 0b11110000) + nibble,
            7 => {
                self.hours = (self.hours & 0b00001111) + ((nibble & 0b0001) << 4);
                self.code_type = match (nibble & 0b0110) >> 1 {
                    0 => TimeCodeType::FPS24,
                    1 => TimeCodeType::FPS25,
                    2 => TimeCodeType::DF30,
                    3 => TimeCodeType::NDF30,
                    _ => panic!("Should not be reachable"),
                }
            }
            _ => panic!("Should not be reachable"),
        }

        frame_number
    }
}

/// Indicates the frame rate of the given [`TimeCode`].
///
/// See [the SMTPE time code standard](https://en.wikipedia.org/wiki/SMPTE_timecode).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeCodeType {
    /// 24 Frames per second
    FPS24 = 0,
    /// 25 Frames per second
    FPS25 = 1,
    /// 30 Frames per second, Drop Frame
    DF30 = 2,
    /// 30 Frames per second, Non-Drop Frame
    NDF30 = 3,
}

impl Default for TimeCodeType {
    fn default() -> Self {
        Self::NDF30
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Like [`TimeCode`] but includes `fractional_frames`. Used in `TimeCodeCueingSetupMsg`.
///
/// As defined in the MIDI Time Code spec (MMA0001 / RP004 / RP008)
pub struct HighResTimeCode {
    /// 0-99
    pub fractional_frames: u8,
    /// 0-29
    pub frames: u8,
    /// 0-59
    pub seconds: u8,
    /// 0-59
    pub minutes: u8,
    /// 0-23
    pub hours: u8,
    pub code_type: TimeCodeType,
}

impl HighResTimeCode {
    /// Return the five byte representation of the frame:
    /// [fractional_frames, frames, seconds, minutes, timecode + hours]
    pub fn to_bytes(self) -> [u8; 5] {
        [
            self.fractional_frames.min(99),
            self.frames.min(29),
            self.seconds.min(59),
            self.minutes.min(59),
            self.hours.min(23) + ((self.code_type as u8) << 5),
        ]
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        let [fractional_frames, frames, seconds, minutes, codehour] = self.to_bytes();
        v.extend_from_slice(&[codehour, minutes, seconds, frames, fractional_frames]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Like [`TimeCode`] but uses `subframes` to optionally include status flags, and fractional frames.
/// Also may be negative. Used in [`MachineControlCommandMsg`](crate::MachineControlCommandMsg).
///
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013) and
/// MIDI Show Control 1.1.1 (RP002/RP014)
pub struct StandardTimeCode {
    pub subframes: SubFrames,
    /// The position in frames, where a negative value indicates a negative TimeCode, -29-29
    pub frames: i8,
    /// The position in seconds, 0-59
    pub seconds: u8,
    /// The position in minutes, 0-59
    pub minutes: u8,
    /// The position in hours, 0-23
    pub hours: u8,
    pub code_type: TimeCodeType,
}

impl StandardTimeCode {
    /// Return the five byte representation of the frame:
    /// [fractional_frames, frames, seconds, minutes, timecode + hours]
    pub fn to_bytes(self) -> [u8; 5] {
        let [subframes, frames] = self.to_bytes_short();
        [
            subframes,
            frames,
            self.seconds.min(59),
            self.minutes.min(59),
            self.hours.min(23) + ((self.code_type as u8) << 5),
        ]
    }

    /// The two byte representation of the frame:
    /// [fractional_frames, frames]
    pub fn to_bytes_short(self) -> [u8; 2] {
        let mut frames = self.frames.abs().min(29) as u8;
        if let SubFrames::Status(_) = self.subframes {
            frames += 1 << 5;
        }
        if self.frames < 0 {
            frames += 1 << 6;
        }
        [self.subframes.to_byte(), frames]
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        let [subframes, frames, seconds, minutes, codehour] = self.to_bytes();
        v.extend_from_slice(&[codehour, minutes, seconds, frames, subframes]);
    }

    #[allow(dead_code)]
    pub(crate) fn extend_midi_short(&self, v: &mut Vec<u8>) {
        let [subframes, frames] = self.to_bytes_short();
        v.extend_from_slice(&[frames, subframes]);
    }
}

impl From<TimeCode> for StandardTimeCode {
    fn from(t: TimeCode) -> Self {
        Self {
            subframes: Default::default(),
            frames: t.frames as i8,
            seconds: t.seconds,
            minutes: t.minutes,
            hours: t.hours,
            code_type: t.code_type,
        }
    }
}

/// Used by [`StandardTimeCode`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubFrames {
    /// The position in fractional frames, 0-99
    FractionalFrames(u8),
    /// Additional flags describing the status of this timecode.
    Status(TimeCodeStatus),
}

impl Default for SubFrames {
    fn default() -> Self {
        Self::FractionalFrames(0)
    }
}

impl SubFrames {
    fn to_byte(&self) -> u8 {
        match *self {
            Self::FractionalFrames(ff) => ff.min(99),
            Self::Status(s) => s.to_byte(),
        }
    }
}

/// Used by [`StandardTimeCode`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeCodeStatus {
    pub estimated_code: bool,
    pub invalid_code: bool,
    pub video_field1: bool,
    pub no_time_code: bool,
}

impl TimeCodeStatus {
    fn to_byte(&self) -> u8 {
        let mut b: u8 = 0;
        if self.estimated_code {
            b += 1 << 6;
        }
        if self.invalid_code {
            b += 1 << 5;
        }
        if self.video_field1 {
            b += 1 << 4;
        }
        if self.no_time_code {
            b += 1 << 3;
        }
        b
    }
}

/// 32 bits defined by SMPTE for "special functions". Used in [`UniversalRealTimeMsg::TimeCodeUserBits`](crate::UniversalRealTimeMsg::TimeCodeUserBits).
/// See [the SMTPE time code standard](https://en.wikipedia.org/wiki/SMPTE_timecode).
///
/// As defined in the MIDI Time Code spec (MMA0001 / RP004 / RP008)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UserBits {
    /// Full bytes can be used here. Sent such that the first is considered
    /// the "most significant" value
    pub bytes: (u8, u8, u8, u8),
    /// SMPTE time code bit 43 (EBU bit 27)
    pub flag1: bool,
    /// SMPTE time code bit 59 (EBU bit 43)
    pub flag2: bool,
}

impl UserBits {
    /// Turn the `UserBits` into its 9 nibble representation:
    /// [nibble_1, nibble_2, nibble_3, nibble_4, nibble_5, nibble_6, nibble_7, nibble_8, nibble_9, nibble_flags]
    pub fn to_nibbles(&self) -> [u8; 9] {
        let [uh, ug] = to_nibble(self.bytes.0);
        let [uf, ue] = to_nibble(self.bytes.1);
        let [ud, uc] = to_nibble(self.bytes.2);
        let [ub, ua] = to_nibble(self.bytes.3);
        let mut flags: u8 = 0;
        if self.flag1 {
            flags += 1;
        }
        if self.flag2 {
            flags += 2;
        }
        [ua, ub, uc, ud, ue, uf, ug, uh, flags]
    }
}

/// Like [`UserBits`] but allows for the embedding of a "secondary time code".
///
/// As defined in MIDI Machine Control 1.0 (MMA0016 / RP013)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StandardUserBits {
    /// Full bytes can be used here. Sent such that the first is considered
    /// the "most significant" value
    pub bytes: (u8, u8, u8, u8),
    /// SMPTE time code bit 43 (EBU bit 27)
    pub flag1: bool,
    /// SMPTE time code bit 59 (EBU bit 43)
    pub flag2: bool,
    /// Contains a secondary time code
    pub secondary_time_code: bool,
}

impl StandardUserBits {
    /// Turn the `UserBits` into its 9 nibble representation:
    /// [nibble_1, nibble_2, nibble_3, nibble_4, nibble_5, nibble_6, nibble_7, nibble_8, nibble_9, nibble_flags]
    pub fn to_nibbles(&self) -> [u8; 9] {
        let [uh, ug] = to_nibble(self.bytes.0);
        let [uf, ue] = to_nibble(self.bytes.1);
        let [ud, uc] = to_nibble(self.bytes.2);
        let [ub, ua] = to_nibble(self.bytes.3);
        let mut flags: u8 = 0;
        if self.flag1 {
            flags += 1;
        }
        if self.flag2 {
            flags += 2;
        }
        if self.secondary_time_code {
            flags += 4;
        }
        [ua, ub, uc, ud, ue, uf, ug, uh, flags]
    }
}

impl From<StandardUserBits> for TimeCode {
    fn from(t: StandardUserBits) -> Self {
        let [ua, ub, uc, ud, ue, uf, ug, uh, _] = t.to_nibbles();
        let frames = (ub << 4) + ua;
        let seconds = (ud << 4) + uc;
        let minutes = (uf << 4) + ue;
        let hours = ((uh & 0b0001) << 4) + ug;
        let code_type = (uh & 0b0110) >> 1;

        TimeCode {
            frames,
            seconds,
            minutes,
            hours,
            code_type: match code_type {
                3 => TimeCodeType::NDF30,
                2 => TimeCodeType::DF30,
                1 => TimeCodeType::FPS25,
                0 => TimeCodeType::FPS24,
                _ => panic!("Should not be reachable"),
            },
        }
    }
}

impl From<TimeCode> for StandardUserBits {
    fn from(t: TimeCode) -> Self {
        let [frame, seconds, minutes, codehour] = t.to_bytes();
        StandardUserBits {
            bytes: (codehour, minutes, seconds, frame),
            flag1: false,
            flag2: false,
            secondary_time_code: true,
        }
    }
}

impl From<UserBits> for StandardUserBits {
    fn from(t: UserBits) -> Self {
        Self {
            bytes: t.bytes,
            flag1: t.flag1,
            flag2: t.flag2,
            secondary_time_code: false,
        }
    }
}

/// Non-realtime Time Code Cueing. Used by [`UniversalNonRealTimeMsg::TimeCodeCueingSetup`](crate::UniversalNonRealTimeMsg::TimeCodeCueingSetup).
///
/// As defined in the MIDI Time Code spec (MMA0001 / RP004 / RP008)
#[derive(Debug, Clone, PartialEq)]
pub enum TimeCodeCueingSetupMsg {
    TimeCodeOffset {
        time_code: HighResTimeCode,
    },
    EnableEventList,
    DisableEventList,
    ClearEventList,
    SystemStop,
    EventListRequest {
        time_code: HighResTimeCode,
    },
    PunchIn {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    PunchOut {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    DeletePunchIn {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    DeletePunchOut {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    EventStart {
        time_code: HighResTimeCode,
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    EventStop {
        time_code: HighResTimeCode,
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    DeleteEventStart {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    DeleteEventStop {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    Cue {
        time_code: HighResTimeCode,
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    DeleteCue {
        time_code: HighResTimeCode,
        event_number: u16,
    },
    EventName {
        time_code: HighResTimeCode,
        event_number: u16,
        name: AsciiString,
    },
}

impl TimeCodeCueingSetupMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::TimeCodeOffset { time_code } => {
                v.push(0x00);
                time_code.extend_midi(v);
                v.push(0x00);
                v.push(0x00);
            }
            Self::EnableEventList => {
                v.push(0x00);
                HighResTimeCode::default().extend_midi(v);
                v.push(0x01);
                v.push(0x00);
            }
            Self::DisableEventList => {
                v.push(0x00);
                HighResTimeCode::default().extend_midi(v);
                v.push(0x02);
                v.push(0x00);
            }
            Self::ClearEventList => {
                v.push(0x00);
                HighResTimeCode::default().extend_midi(v);
                v.push(0x03);
                v.push(0x00);
            }
            Self::SystemStop => {
                v.push(0x00);
                HighResTimeCode::default().extend_midi(v);
                v.push(0x04);
                v.push(0x00);
            }
            Self::EventListRequest { time_code } => {
                v.push(0x00);
                time_code.extend_midi(v);
                v.push(0x05);
                v.push(0x00);
            }
            Self::PunchIn {
                time_code,
                event_number,
            } => {
                v.push(0x01);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::PunchOut {
                time_code,
                event_number,
            } => {
                v.push(0x02);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::DeletePunchIn {
                time_code,
                event_number,
            } => {
                v.push(0x03);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::DeletePunchOut {
                time_code,
                event_number,
            } => {
                v.push(0x04);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::EventStart {
                time_code,
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x05);
                } else {
                    v.push(0x07);
                }
                time_code.extend_midi(v);
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::EventStop {
                time_code,
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x06);
                } else {
                    v.push(0x08);
                }
                time_code.extend_midi(v);
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::DeleteEventStart {
                time_code,
                event_number,
            } => {
                v.push(0x09);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::DeleteEventStop {
                time_code,
                event_number,
            } => {
                v.push(0x0A);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::Cue {
                time_code,
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x0B);
                } else {
                    v.push(0x0C);
                }
                time_code.extend_midi(v);
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::DeleteCue {
                time_code,
                event_number,
            } => {
                v.push(0x0D);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
            }
            Self::EventName {
                time_code,
                event_number,
                name,
            } => {
                v.push(0x0E);
                time_code.extend_midi(v);
                push_u14(*event_number, v);
                push_nibblized_name(name, v);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

/// Realtime Time Code Cueing. Used by [`UniversalRealTimeMsg::TimeCodeCueing`](crate::UniversalRealTimeMsg::TimeCodeCueing).
///
/// As defined in the MIDI Time Code spec (MMA0001 / RP004 / RP008)
#[derive(Debug, Clone, PartialEq)]
pub enum TimeCodeCueingMsg {
    SystemStop,
    PunchIn {
        event_number: u16,
    },
    PunchOut {
        event_number: u16,
    },
    EventStart {
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    EventStop {
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    Cue {
        event_number: u16,
        additional_information: Vec<MidiMsg>,
    },
    EventName {
        event_number: u16,
        name: AsciiString,
    },
}

fn push_nibblized_midi(msgs: &[MidiMsg], v: &mut Vec<u8>) {
    for msg in msgs.iter() {
        for b in msg.to_midi().iter() {
            let [msn, lsn] = to_nibble(*b);
            v.push(lsn);
            v.push(msn);
        }
    }
}

fn push_nibblized_name(name: &AsciiString, v: &mut Vec<u8>) {
    // Not sure if this actually handles newlines correctly
    for b in name.as_bytes().iter() {
        let [msn, lsn] = to_nibble(*b);
        v.push(lsn);
        v.push(msn);
    }
}

impl TimeCodeCueingMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::SystemStop => {
                v.push(0x00);
                v.push(0x04);
                v.push(0x00);
            }
            Self::PunchIn { event_number } => {
                v.push(0x01);
                push_u14(*event_number, v);
            }
            Self::PunchOut { event_number } => {
                v.push(0x02);
                push_u14(*event_number, v);
            }
            Self::EventStart {
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x05);
                } else {
                    v.push(0x07);
                }
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::EventStop {
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x06);
                } else {
                    v.push(0x08);
                }
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::Cue {
                event_number,
                additional_information,
            } => {
                if additional_information.is_empty() {
                    v.push(0x0B);
                } else {
                    v.push(0x0C);
                }
                push_u14(*event_number, v);
                push_nibblized_midi(additional_information, v);
            }
            Self::EventName { event_number, name } => {
                v.push(0x0E);
                push_u14(*event_number, v);
                push_nibblized_name(name, v);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn serialize_time_code_cuing_setup_msg() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalNonRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalNonRealTimeMsg::TimeCodeCueingSetup(
                        TimeCodeCueingSetupMsg::SystemStop
                    ),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7E, 0x7f, 04, 00, 96, 00, 00, 00, 00, 04, 00, 0xF7]
        );
    }
    #[test]
    fn serialize_time_code_cuing_msg() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeCodeCueing(TimeCodeCueingMsg::SystemStop),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 05, 00, 04, 00, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeCodeCueing(TimeCodeCueingMsg::EventStart {
                        event_number: 511,
                        additional_information: vec![]
                    }),
                },
            }
            .to_midi(),
            vec![0xF0, 0x7F, 0x7f, 05, 05, 0x7f, 0x03, 0xF7]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::TimeCodeCueing(TimeCodeCueingMsg::EventStart {
                        event_number: 511,
                        additional_information: vec![MidiMsg::ChannelVoice {
                            channel: Channel::Ch2,
                            msg: ChannelVoiceMsg::NoteOn {
                                note: 0x55,
                                velocity: 0x67
                            }
                        }]
                    }),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7f, 05, 07, 0x7f, 0x03,
                // Note on midi msg: 0x91, 0x55, 0x67
                0x01, 0x09, 0x05, 0x05, 0x07, 0x06, // End
                0xF7
            ]
        );
    }
}
