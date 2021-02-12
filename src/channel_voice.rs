use super::util::*;

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
    pub fn to_midi(self) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::super::*;

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
}
