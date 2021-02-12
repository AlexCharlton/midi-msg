use num_derive::FromPrimitive;

use super::time_code::*;
use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MidiMsg {
    ChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    RunningChannelVoice {
        channel: Channel,
        msg: ChannelVoiceMsg,
    },
    ChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    RunningChannelMode {
        channel: Channel,
        msg: ChannelModeMsg,
    },
    SystemCommon {
        msg: SystemCommonMsg,
    },
    SystemRealTime {
        msg: SystemRealTimeMsg,
    },
    SystemExclusive {
        msg: SystemExclusiveMsg,
    },
}

impl MidiMsg {
    pub fn to_midi(self) -> Vec<u8> {
        self.into()
    }

    pub fn from_midi(_m: &[u8]) -> Result<Self, &str> {
        Err("TODO: not implemented")
    }
}

impl From<MidiMsg> for Vec<u8> {
    fn from(m: MidiMsg) -> Vec<u8> {
        match m {
            MidiMsg::ChannelVoice { channel, msg } => {
                let mut r = msg.to_midi();
                r[0] += channel as u8;
                r
            }
            MidiMsg::RunningChannelVoice { msg, .. } => msg.to_midi_running(),
            MidiMsg::ChannelMode { channel, msg } => {
                let mut r = msg.to_midi();
                r[0] += channel as u8;
                r
            }
            MidiMsg::RunningChannelMode { msg, .. } => msg.to_midi_running(),
            MidiMsg::SystemCommon { msg } => msg.to_midi(),
            MidiMsg::SystemRealTime { msg } => msg.to_midi(),
            MidiMsg::SystemExclusive { msg } => msg.to_midi(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelVoiceMsg {
    /// Max values are 127
    NoteOff {
        note: u8,
        velocity: u8,
    },
    /// Max values are 127
    NoteOn {
        note: u8,
        velocity: u8,
    },
    /// Max values are 127
    PolyPressure {
        note: u8,
        pressure: u8,
    },
    ControlChange {
        control: ControlChange,
    },
    /// Max 127
    ProgramChange {
        program: u8,
    },
    /// Max 127
    ChannelPressure {
        pressure: u8,
    },
    /// Max 16383
    PitchBend {
        bend: u16,
    },
}

impl ChannelVoiceMsg {
    pub fn to_midi(self) -> Vec<u8> {
        match self {
            ChannelVoiceMsg::NoteOff { note, velocity } => vec![0x80, to_u7(note), to_u7(velocity)],
            ChannelVoiceMsg::NoteOn { note, velocity } => vec![0x90, to_u7(note), to_u7(velocity)],
            ChannelVoiceMsg::PolyPressure { note, pressure } => {
                vec![0xA0, to_u7(note), to_u7(pressure)]
            }
            ChannelVoiceMsg::ControlChange { control } => {
                let mut r: Vec<u8> = vec![0xB0];
                r.extend(control.to_midi());
                r
            }
            ChannelVoiceMsg::ProgramChange { program } => vec![0xC0, to_u7(program)],
            ChannelVoiceMsg::ChannelPressure { pressure } => vec![0xD0, to_u7(pressure)],
            ChannelVoiceMsg::PitchBend { bend } => {
                let [msb, lsb] = to_u14(bend);
                vec![0xE0, lsb, msb]
            }
        }
    }

    pub fn to_midi_running(self) -> Vec<u8> {
        match self {
            ChannelVoiceMsg::NoteOff { note, velocity } => vec![to_u7(note), to_u7(velocity)],
            ChannelVoiceMsg::NoteOn { note, velocity } => vec![to_u7(note), to_u7(velocity)],
            ChannelVoiceMsg::PolyPressure { note, pressure } => vec![to_u7(note), to_u7(pressure)],
            ChannelVoiceMsg::ControlChange { control } => control.to_midi(),
            ChannelVoiceMsg::ProgramChange { program } => vec![to_u7(program)],
            ChannelVoiceMsg::ChannelPressure { pressure } => vec![to_u7(pressure)],
            ChannelVoiceMsg::PitchBend { bend } => {
                let [msb, lsb] = to_u14(bend);
                vec![lsb, msb]
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlChange {
    /// Max 16383
    BankSelect(u16),
    /// Max 16383
    ModWheel(u16),
    /// Max 16383
    Breath(u16),
    /// Control number may be any valid Midi CC Control number. May not be > 119.
    /// Max value is 127
    Undefined {
        control: u8,
        value: u8,
    },
    /// `control1` is associated with the MSB, `control2` with the LSB. Neither may be > 119.
    /// Max value is 16383
    UndefinedHighRes {
        control1: u8,
        control2: u8,
        value: u16,
    },
    /// Max 16383
    Foot(u16),
    /// Max 16383
    Portamento(u16),
    /// Max 16383
    Volume(u16),
    /// Max 16383
    Balance(u16),
    /// Max 16383
    Pan(u16),
    /// Max 16383
    Expression(u16),
    /// Max 16383
    Effect1(u16),
    /// Max 16383
    Effect2(u16),
    /// Max 16383
    GeneralPurpose1(u16),
    /// Max 16383
    GeneralPurpose2(u16),
    /// Max 16383
    GeneralPurpose3(u16),
    /// Max 16383
    GeneralPurpose4(u16),
    /// Max 127
    GeneralPurpose5(u8),
    /// Max 127
    GeneralPurpose6(u8),
    /// Max 127
    GeneralPurpose7(u8),
    /// Max 127
    GeneralPurpose8(u8),
    /// Max 127
    Hold(u8),
    /// Max 127
    Hold2(u8),
    TogglePortamento(bool),
    /// Max 127
    Sostenuto(u8),
    /// Max 127
    SoftPedal(u8),
    ToggleLegato(bool),
    /// Max 127
    SoundVariation(u8),
    /// Max 127
    Timbre(u8),
    /// Max 127
    ReleaseTime(u8),
    /// Max 127
    AttachTime(u8),
    /// Max 127
    Brightness(u8),
    /// Max 127
    SoundControl6(u8),
    /// Max 127
    SoundControl7(u8),
    /// Max 127
    SoundControl8(u8),
    /// Max 127
    SoundControl9(u8),
    /// Max 127
    SoundControl10(u8),
    /// Max 127
    PortamentoControl(u8),
    /// Max 127
    ExternalFX(u8),
    /// Max 127
    Tremolo(u8),
    /// Max 127
    Chorus(u8),
    /// Max 127
    Celeste(u8),
    /// Max 127
    Phaser(u8),

    // Parameters
    Parameter(Parameter),
    /// Max 16383
    DataEntry(u16),
    /// Max 127
    DataIncrement(u8),
    /// Max 127
    DataDecrement(u8),
}

impl ControlChange {
    fn high_res_cc(control: u8, value: u16) -> Vec<u8> {
        let [msb, lsb] = to_u14(value);
        vec![control, msb, control + 32, lsb]
    }

    fn undefined(control: u8, value: u8) -> Vec<u8> {
        vec![control.min(119), to_u7(value)]
    }

    fn undefined_high_res(control1: u8, control2: u8, value: u16) -> Vec<u8> {
        let [msb, lsb] = to_u14(value);
        vec![control1.min(119), msb, control2.min(119), lsb]
    }
}

impl ControlChange {
    fn to_midi(self) -> Vec<u8> {
        match self {
            ControlChange::BankSelect(x) => ControlChange::high_res_cc(0, x),
            ControlChange::ModWheel(x) => ControlChange::high_res_cc(1, x),
            ControlChange::Breath(x) => ControlChange::high_res_cc(2, x),
            ControlChange::Undefined { control, value } => ControlChange::undefined(control, value),
            ControlChange::UndefinedHighRes {
                control1,
                control2,
                value,
            } => ControlChange::undefined_high_res(control1, control2, value),
            ControlChange::Foot(x) => ControlChange::high_res_cc(4, x),
            ControlChange::Portamento(x) => ControlChange::high_res_cc(5, x),
            ControlChange::Volume(x) => ControlChange::high_res_cc(7, x),
            ControlChange::Balance(x) => ControlChange::high_res_cc(8, x),
            ControlChange::Pan(x) => ControlChange::high_res_cc(10, x),
            ControlChange::Expression(x) => ControlChange::high_res_cc(11, x),
            ControlChange::Effect1(x) => ControlChange::high_res_cc(12, x),
            ControlChange::Effect2(x) => ControlChange::high_res_cc(13, x),
            ControlChange::GeneralPurpose1(x) => ControlChange::high_res_cc(0, x),
            ControlChange::GeneralPurpose2(x) => ControlChange::high_res_cc(0, x),
            ControlChange::GeneralPurpose3(x) => ControlChange::high_res_cc(0, x),
            ControlChange::GeneralPurpose4(x) => ControlChange::high_res_cc(0, x),
            ControlChange::GeneralPurpose5(x) => vec![80, to_u7(x)],
            ControlChange::GeneralPurpose6(x) => vec![82, to_u7(x)],
            ControlChange::GeneralPurpose7(x) => vec![83, to_u7(x)],
            ControlChange::GeneralPurpose8(x) => vec![84, to_u7(x)],
            ControlChange::Hold(x) => vec![64, to_u7(x)],
            ControlChange::Hold2(x) => vec![69, to_u7(x)],
            ControlChange::TogglePortamento(on) => vec![65, if on { 127 } else { 0 }],
            ControlChange::Sostenuto(x) => vec![66, to_u7(x)],
            ControlChange::SoftPedal(x) => vec![67, to_u7(x)],
            ControlChange::ToggleLegato(on) => vec![68, if on { 127 } else { 0 }],
            ControlChange::SoundVariation(x) => vec![70, to_u7(x)],
            ControlChange::Timbre(x) => vec![71, to_u7(x)],
            ControlChange::ReleaseTime(x) => vec![72, to_u7(x)],
            ControlChange::AttachTime(x) => vec![73, to_u7(x)],
            ControlChange::Brightness(x) => vec![74, to_u7(x)],
            ControlChange::SoundControl6(x) => vec![75, to_u7(x)],
            ControlChange::SoundControl7(x) => vec![76, to_u7(x)],
            ControlChange::SoundControl8(x) => vec![77, to_u7(x)],
            ControlChange::SoundControl9(x) => vec![78, to_u7(x)],
            ControlChange::SoundControl10(x) => vec![79, to_u7(x)],
            ControlChange::PortamentoControl(x) => vec![84, to_u7(x)],
            ControlChange::ExternalFX(x) => vec![91, to_u7(x)],
            ControlChange::Tremolo(x) => vec![92, to_u7(x)],
            ControlChange::Chorus(x) => vec![93, to_u7(x)],
            ControlChange::Celeste(x) => vec![94, to_u7(x)],
            ControlChange::Phaser(x) => vec![95, to_u7(x)],

            // Parameters
            ControlChange::Parameter(p) => p.into(),
            ControlChange::DataEntry(x) => ControlChange::high_res_cc(6, x),
            ControlChange::DataIncrement(x) => vec![96, to_u7(x)],
            ControlChange::DataDecrement(x) => vec![97, to_u7(x)],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parameter {
    PitchBendSensitivity,
    FineTuning,
    CoarseTuning,
    TuningProgramSelect,
    TuningBankSelect,
    /// 0-16383
    Unregistered(u16),
}

impl From<Parameter> for Vec<u8> {
    fn from(p: Parameter) -> Vec<u8> {
        match p {
            Parameter::PitchBendSensitivity => vec![100, 0, 101, 0],
            Parameter::FineTuning => vec![100, 1, 101, 0],
            Parameter::CoarseTuning => vec![100, 2, 101, 0],
            Parameter::TuningProgramSelect => vec![100, 3, 101, 0],
            Parameter::TuningBankSelect => vec![100, 4, 101, 0],
            Parameter::Unregistered(x) => {
                let [msb, lsb] = to_u14(x);
                vec![98, lsb, 99, msb]
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelModeMsg {
    AllSoundOff,
    ResetAllControllers,
    LocalControl(bool),
    AllNotesOff,
    OmniMode(bool),
    PolyMode(PolyMode),
}

impl ChannelModeMsg {
    fn to_midi(self) -> Vec<u8> {
        let m = self.to_midi_running();
        vec![0xB0, m[0], m[1]]
    }

    fn to_midi_running(self) -> Vec<u8> {
        match self {
            ChannelModeMsg::AllSoundOff => vec![120, 0],
            ChannelModeMsg::ResetAllControllers => vec![121, 0],
            ChannelModeMsg::LocalControl(on) => vec![122, if on { 127 } else { 0 }],
            ChannelModeMsg::AllNotesOff => vec![123, 0],
            ChannelModeMsg::OmniMode(on) => vec![if on { 125 } else { 124 }, 0],
            ChannelModeMsg::PolyMode(m) => vec![
                if m == PolyMode::Poly { 127 } else { 126 },
                match m {
                    PolyMode::Poly => 0,
                    PolyMode::Mono(n) => to_u7(n),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolyMode {
    Mono(u8),
    Poly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemCommonMsg {
    MTCQuarterFrame1(Frame),
    MTCQuarterFrame2(Frame),
    MTCQuarterFrame3(Frame),
    MTCQuarterFrame4(Frame),
    MTCQuarterFrame5(Frame),
    MTCQuarterFrame6(Frame),
    MTCQuarterFrame7(Frame),
    MTCQuarterFrame8(Frame),
    /// Max 16383
    SongPosition(u16),
    /// Max 127
    SongSelect(u8),
    TuneRequest,
}

impl SystemCommonMsg {
    fn to_midi(self) -> Vec<u8> {
        self.into()
    }
}

impl From<SystemCommonMsg> for Vec<u8> {
    fn from(m: SystemCommonMsg) -> Vec<u8> {
        match m {
            SystemCommonMsg::MTCQuarterFrame1(qf) => vec![0xF1, qf.to_nibbles()[0]],
            SystemCommonMsg::MTCQuarterFrame2(qf) => vec![0xF1, qf.to_nibbles()[1]],
            SystemCommonMsg::MTCQuarterFrame3(qf) => vec![0xF1, qf.to_nibbles()[2]],
            SystemCommonMsg::MTCQuarterFrame4(qf) => vec![0xF1, qf.to_nibbles()[3]],
            SystemCommonMsg::MTCQuarterFrame5(qf) => vec![0xF1, qf.to_nibbles()[4]],
            SystemCommonMsg::MTCQuarterFrame6(qf) => vec![0xF1, qf.to_nibbles()[5]],
            SystemCommonMsg::MTCQuarterFrame7(qf) => vec![0xF1, qf.to_nibbles()[6]],
            SystemCommonMsg::MTCQuarterFrame8(qf) => vec![0xF1, qf.to_nibbles()[7]],
            SystemCommonMsg::SongPosition(pos) => {
                let [msb, lsb] = to_u14(pos);
                vec![0xF2, lsb, msb]
            }
            SystemCommonMsg::SongSelect(song) => vec![0xF3, to_u7(song)],
            SystemCommonMsg::TuneRequest => vec![0xF6],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemRealTimeMsg {
    // TODO
}

impl SystemRealTimeMsg {
    fn to_midi(self) -> Vec<u8> {
        self.into()
    }
}

impl From<SystemRealTimeMsg> for Vec<u8> {
    fn from(_m: SystemRealTimeMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemExclusiveMsg {
    // TODO
}

impl SystemExclusiveMsg {
    fn to_midi(self) -> Vec<u8> {
        self.into()
    }
}

impl From<SystemExclusiveMsg> for Vec<u8> {
    fn from(_m: SystemExclusiveMsg) -> Vec<u8> {
        vec![] // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_u7() {
        assert_eq!(to_u7(0xff), 127); // Overflow is treated as max value
        assert_eq!(to_u7(0x77), 0x77);
        assert_eq!(to_u7(0x00), 0x00);
        assert_eq!(to_u7(0x7f), 127);
    }

    #[test]
    fn test_to_u14() {
        assert_eq!(to_u14(0xff), [1, 127]);
        assert_eq!(to_u14(0xffff), [127, 127]); // Overflow is treated as max value
        assert_eq!(to_u14(0x00), [0, 0]);
        assert_eq!(to_u14(0xfff), [0x1f, 127]);
        assert_eq!(to_u14(1000), [0x07, 0x68]);
    }

    #[test]
    fn serialize_channel_voice_msg() {
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

        assert_eq!(
            MidiMsg::RunningChannelVoice {
                channel: Channel::Ch10,
                msg: ChannelVoiceMsg::PitchBend { bend: 0xff44 }
            }
            .to_midi(),
            vec![0x44, 0x7f]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch10,
                msg: ChannelVoiceMsg::PitchBend { bend: 0xff44 }
            }
            .to_midi(),
            vec![0xE9, 0x44, 0x7f]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Volume(1000)
                }
            }
            .to_midi(),
            vec![0xB1, 7, 0x07, 39, 0x68]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch4,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Undefined {
                        control: 85,
                        value: 77
                    }
                }
            }
            .to_midi(),
            vec![0xB3, 85, 77]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::UndefinedHighRes {
                        control1: 3,
                        control2: 35,
                        value: 1000
                    }
                }
            }
            .to_midi(),
            vec![0xB1, 3, 0x07, 35, 0x68]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch3,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::TogglePortamento(true)
                }
            }
            .to_midi(),
            vec![0xB2, 65, 0x7f]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::FineTuning)
                }
            }
            .to_midi(),
            vec![0xB1, 100, 0x01, 101, 0x00]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::Unregistered(1000))
                }
            }
            .to_midi(),
            vec![0xB1, 98, 0x68, 99, 0x07]
        );
    }

    #[test]
    fn serialize_channel_mode_msg() {
        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![0xB2, 120, 0]
        );

        assert_eq!(
            MidiMsg::RunningChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::AllSoundOff
            }
            .to_midi(),
            vec![120, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::LocalControl(true)
            }
            .to_midi(),
            vec![0xB2, 122, 127]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(true)
            }
            .to_midi(),
            vec![0xB2, 125, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::OmniMode(false)
            }
            .to_midi(),
            vec![0xB2, 124, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Poly)
            }
            .to_midi(),
            vec![0xB2, 127, 0]
        );

        assert_eq!(
            MidiMsg::ChannelMode {
                channel: Channel::Ch3,
                msg: ChannelModeMsg::PolyMode(PolyMode::Mono(4))
            }
            .to_midi(),
            vec![0xB2, 126, 4]
        );
    }

    #[test]
    fn serialize_system_common_msg() {
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TuneRequest
            }
            .to_midi(),
            vec![0xF6]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongSelect(69)
            }
            .to_midi(),
            vec![0xF3, 69]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongPosition(1000)
            }
            .to_midi(),
            vec![0xF2, 0x68, 0x07]
        );

        let frame = Frame {
            frame: 40,                     // Should be limited to 29: 0b00011101
            seconds: 58,                   // 0b00111010
            minutes: 20,                   // 0b00010100
            hours: 25,                     // Should be limited to 23: 0b00010111
            code_type: TimeCodeType::DF30, //      0b01000000
        };

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame1(frame)
            }
            .to_midi(),
            vec![0xF1, 0b1101]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame2(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame3(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00100000 + 0b1010]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame4(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00110000 + 0b0011]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame5(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01000000 + 0b0100]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame6(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame7(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01100000 + 0b0111]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame8(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01110000 + 0b0101]
        );
    }
}
