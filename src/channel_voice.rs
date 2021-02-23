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
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn to_midi_running(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi_running(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            ChannelVoiceMsg::NoteOff { .. } => v.push(0x80),
            ChannelVoiceMsg::NoteOn { .. } => v.push(0x90),
            ChannelVoiceMsg::PolyPressure { .. } => v.push(0xA0),
            ChannelVoiceMsg::ControlChange { .. } => v.push(0xB0),
            ChannelVoiceMsg::ProgramChange { .. } => v.push(0xC0),
            ChannelVoiceMsg::ChannelPressure { .. } => v.push(0xD0),
            ChannelVoiceMsg::PitchBend { .. } => v.push(0xE0),
        }
        self.extend_midi_running(v);
    }

    pub fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match *self {
            ChannelVoiceMsg::NoteOff { note, velocity } => {
                v.push(to_u7(note));
                v.push(to_u7(velocity));
            }
            ChannelVoiceMsg::NoteOn { note, velocity } => {
                v.push(to_u7(note));
                v.push(to_u7(velocity));
            }
            ChannelVoiceMsg::PolyPressure { note, pressure } => {
                v.push(to_u7(note));
                v.push(to_u7(pressure));
            }
            ChannelVoiceMsg::ControlChange { control } => control.extend_midi_running(v),
            ChannelVoiceMsg::ProgramChange { program } => v.push(to_u7(program)),
            ChannelVoiceMsg::ChannelPressure { pressure } => v.push(to_u7(pressure)),
            ChannelVoiceMsg::PitchBend { bend } => {
                push_u14(bend, v);
            }
        }
    }

    /// Ok results return a MidiMsg and the number of bytes consumed from the input
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

impl From<&ChannelVoiceMsg> for Vec<u8> {
    fn from(m: &ChannelVoiceMsg) -> Vec<u8> {
        m.to_midi()
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

    /// Registered and Unregistered Parameters
    Parameter(Parameter),
    /// Set the value of the last-set Parameter 0-16383
    DataEntry(u16),
    /// Set the MSB and LSB of the last-set parameter separately
    DataEntry2(u8, u8),
    /// Increment the value of the last-set Parameter 0-127
    DataIncrement(u8),
    /// Decrement the value of the last-set Parameter 0-127
    DataDecrement(u8),
}

impl ControlChange {
    fn high_res_cc(v: &mut Vec<u8>, control: u8, value: u16) {
        let [msb, lsb] = to_u14(value);
        v.push(control);
        v.push(msb);
        v.push(control + 32);
        v.push(lsb);
    }

    fn undefined(v: &mut Vec<u8>, control: u8, value: u8) {
        v.push(control.min(119));
        v.push(to_u7(value));
    }

    fn undefined_high_res(v: &mut Vec<u8>, control1: u8, control2: u8, value: u16) {
        let [msb, lsb] = to_u14(value);
        v.push(control1.min(119));
        v.push(msb);
        v.push(control2.min(119));
        v.push(lsb);
    }
}

impl ControlChange {
    pub fn to_midi_running(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi_running(&mut r);
        r
    }

    pub fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match *self {
            ControlChange::BankSelect(x) => ControlChange::high_res_cc(v, 0, x),
            ControlChange::ModWheel(x) => ControlChange::high_res_cc(v, 1, x),
            ControlChange::Breath(x) => ControlChange::high_res_cc(v, 2, x),
            ControlChange::Undefined { control, value } => {
                ControlChange::undefined(v, control, value)
            }
            ControlChange::UndefinedHighRes {
                control1,
                control2,
                value,
            } => ControlChange::undefined_high_res(v, control1, control2, value),
            ControlChange::Foot(x) => ControlChange::high_res_cc(v, 4, x),
            ControlChange::Portamento(x) => ControlChange::high_res_cc(v, 5, x),
            ControlChange::Volume(x) => ControlChange::high_res_cc(v, 7, x),
            ControlChange::Balance(x) => ControlChange::high_res_cc(v, 8, x),
            ControlChange::Pan(x) => ControlChange::high_res_cc(v, 10, x),
            ControlChange::Expression(x) => ControlChange::high_res_cc(v, 11, x),
            ControlChange::Effect1(x) => ControlChange::high_res_cc(v, 12, x),
            ControlChange::Effect2(x) => ControlChange::high_res_cc(v, 13, x),
            ControlChange::GeneralPurpose1(x) => ControlChange::high_res_cc(v, 0, x),
            ControlChange::GeneralPurpose2(x) => ControlChange::high_res_cc(v, 0, x),
            ControlChange::GeneralPurpose3(x) => ControlChange::high_res_cc(v, 0, x),
            ControlChange::GeneralPurpose4(x) => ControlChange::high_res_cc(v, 0, x),
            ControlChange::GeneralPurpose5(x) => {
                v.push(80);
                v.push(to_u7(x));
            }
            ControlChange::GeneralPurpose6(x) => {
                v.push(82);
                v.push(to_u7(x));
            }
            ControlChange::GeneralPurpose7(x) => {
                v.push(83);
                v.push(to_u7(x));
            }
            ControlChange::GeneralPurpose8(x) => {
                v.push(84);
                v.push(to_u7(x));
            }
            ControlChange::Hold(x) => {
                v.push(64);
                v.push(to_u7(x));
            }
            ControlChange::Hold2(x) => {
                v.push(69);
                v.push(to_u7(x));
            }
            ControlChange::TogglePortamento(on) => {
                v.push(65);
                v.push(if on { 127 } else { 0 });
            }
            ControlChange::Sostenuto(x) => {
                v.push(66);
                v.push(to_u7(x));
            }
            ControlChange::SoftPedal(x) => {
                v.push(67);
                v.push(to_u7(x));
            }
            ControlChange::ToggleLegato(on) => {
                v.push(68);
                v.push(if on { 127 } else { 0 });
            }
            ControlChange::SoundVariation(x) => {
                v.push(70);
                v.push(to_u7(x));
            }
            ControlChange::Timbre(x) => {
                v.push(71);
                v.push(to_u7(x));
            }
            ControlChange::ReleaseTime(x) => {
                v.push(72);
                v.push(to_u7(x));
            }
            ControlChange::AttachTime(x) => {
                v.push(73);
                v.push(to_u7(x));
            }
            ControlChange::Brightness(x) => {
                v.push(74);
                v.push(to_u7(x));
            }
            ControlChange::SoundControl6(x) => {
                v.push(75);
                v.push(to_u7(x));
            }
            ControlChange::SoundControl7(x) => {
                v.push(76);
                v.push(to_u7(x));
            }
            ControlChange::SoundControl8(x) => {
                v.push(77);
                v.push(to_u7(x));
            }
            ControlChange::SoundControl9(x) => {
                v.push(78);
                v.push(to_u7(x));
            }
            ControlChange::SoundControl10(x) => {
                v.push(79);
                v.push(to_u7(x));
            }
            ControlChange::PortamentoControl(x) => {
                v.push(84);
                v.push(to_u7(x));
            }
            ControlChange::ExternalFX(x) => {
                v.push(91);
                v.push(to_u7(x));
            }
            ControlChange::Tremolo(x) => {
                v.push(92);
                v.push(to_u7(x));
            }
            ControlChange::Chorus(x) => {
                v.push(93);
                v.push(to_u7(x));
            }
            ControlChange::Celeste(x) => {
                v.push(94);
                v.push(to_u7(x));
            }
            ControlChange::Phaser(x) => {
                v.push(95);
                v.push(to_u7(x));
            }

            // Parameters
            ControlChange::Parameter(p) => p.extend_midi_running(v),
            ControlChange::DataEntry(x) => ControlChange::high_res_cc(v, 6, x),
            ControlChange::DataEntry2(msb, lsb) => {
                v.push(6);
                v.push(msb);
                v.push(6 + 32);
                v.push(lsb);
            }
            ControlChange::DataIncrement(x) => {
                v.push(96);
                v.push(to_u7(x));
            }
            ControlChange::DataDecrement(x) => {
                v.push(97);
                v.push(to_u7(x));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// "Entry" Parameters can be used to set the given parameters
pub enum Parameter {
    /// The pitch bend sensitivity in semitones (0-127) and the sensitivity in cents (0-100),
    /// respectively. For example, a value (1, 0) means +/- one semitone (a total range of two
    /// semitones)
    PitchBendSensitivity,
    PitchBendSensitivityEntry(u8, u8),
    /// A value from -8192-8191, representing the fractional cents to shift away from A440
    /// in 1/8192ths of a cent
    FineTuning,
    FineTuningEntry(i16),
    /// A value from -64-63, the number of semitones to shift away from A44
    CoarseTuning,
    CoarseTuningEntry(i8),
    /// Which "Tuning Program" to select from: 0-127
    /// Defined in the MIDI Tuning Standard (Updated Specification)
    TuningProgramSelect,
    TuningProgramSelectEntry(u8),
    /// Which "Tuning Bank" to select from: 0-127
    /// Defined in the MIDI Tuning Standard (Updated Specification)
    TuningBankSelect,
    TuningBankSelectEntry(u8),
    /// The amount of "modulation depth" your mod wheel should apply: 0-16383
    /// There's no firm definition of what this means. Defined in CA 26
    ModulationDepthRange,
    ModulationDepthRangeEntry(u16),
    /// Only valid when sent to channel 1 or channel 16, the former indicating that this
    /// is configuring the number of "lower zone" channels and the latter referring to the
    /// "upper zone". A value between 0 (zone is not configured to be MPE) and 16 (zone has
    /// 16 channels in it). There can be no more than lower zone channels + upper zone channels
    /// active at a given time.
    /// Defined in RP-053: MIDI Polyphonic Expression
    PolyphonicExpression,
    PolyphonicExpressionEntry(u8),
    /// 0-16383
    Unregistered(u16),
}

impl Parameter {
    fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match self {
            Parameter::PitchBendSensitivity => {
                v.push(100);
                v.push(0);
                v.push(101);
                v.push(0);
            }
            Parameter::PitchBendSensitivityEntry(c, f) => {
                v.push(100);
                v.push(0);
                v.push(101);
                v.push(0);
                // Data entry:
                v.push(6);
                v.push(*c);
                v.push(6 + 32);
                v.push((*f).min(100));
            }
            Parameter::FineTuning => {
                v.push(100);
                v.push(1);
                v.push(101);
                v.push(0);
            }
            Parameter::FineTuningEntry(x) => {
                v.push(100);
                v.push(1);
                v.push(101);
                v.push(0);
                // Data entry:
                let [msb, lsb] = to_i14(*x);
                v.push(6);
                v.push(msb);
                v.push(6 + 32);
                v.push(lsb);
            }
            Parameter::CoarseTuning => {
                v.push(100);
                v.push(2);
                v.push(101);
                v.push(0);
            }
            Parameter::CoarseTuningEntry(x) => {
                v.push(100);
                v.push(2);
                v.push(101);
                v.push(0);
                // Data entry:
                let lsb = to_i7(*x);
                v.push(6);
                v.push(0);
                v.push(6 + 32);
                v.push(lsb);
            }
            Parameter::TuningProgramSelect => {
                v.push(100);
                v.push(3);
                v.push(101);
                v.push(0);
            }
            Parameter::TuningProgramSelectEntry(x) => {
                v.push(100);
                v.push(3);
                v.push(101);
                v.push(0);
                // Data entry (MSB only)
                v.push(6);
                v.push(*x);
            }
            Parameter::TuningBankSelect => {
                v.push(100);
                v.push(4);
                v.push(101);
                v.push(0);
            }
            Parameter::TuningBankSelectEntry(x) => {
                v.push(100);
                v.push(3);
                v.push(101);
                v.push(0);
                // Data entry (MSB only)
                v.push(6);
                v.push(*x);
            }
            Parameter::ModulationDepthRange => {
                v.push(100);
                v.push(5);
                v.push(101);
                v.push(0);
            }
            Parameter::ModulationDepthRangeEntry(x) => {
                v.push(100);
                v.push(5);
                v.push(101);
                v.push(0);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Parameter::PolyphonicExpression => {
                v.push(100);
                v.push(6);
                v.push(101);
                v.push(0);
            }
            Parameter::PolyphonicExpressionEntry(x) => {
                v.push(100);
                v.push(6);
                v.push(101);
                v.push(0);
                // Data entry (MSB only)
                v.push(6);
                v.push((*x).min(16));
            }
            Parameter::Unregistered(x) => {
                let [msb, lsb] = to_u14(*x);
                v.push(98);
                v.push(lsb);
                v.push(99);
                v.push(msb);
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
            vec![0x7f, 0x7f]
        );

        assert_eq!(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch10,
                msg: ChannelVoiceMsg::PitchBend { bend: 1000 }
            }
            .to_midi(),
            vec![0xE9, 0x68, 0x07]
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
