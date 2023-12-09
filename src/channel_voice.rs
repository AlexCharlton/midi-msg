use super::parse_error::*;
use super::util::*;
use alloc::vec::Vec;
use alloc::vec;

/// Channel-level messages that act on a voice. For instance, turning notes on off,
/// or modifying sounding notes. Used in [`MidiMsg`](crate::MidiMsg).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelVoiceMsg {
    /// Turn on a note
    NoteOn {
        /// A MIDI note number 0-127. Per GM1, 69 = A440
        note: u8,
        /// The velocity the note should be played at, 0-127
        velocity: u8,
    },
    /// Turn off a note
    NoteOff {
        /// Stop playing the given MIDI note at this channel, 0-127
        note: u8,
        /// The velocity the note should stop being played at, 0-127
        velocity: u8,
    },
    /// Generally used for modifying the tones being played. Frequently shortened to 'CC'
    ControlChange { control: ControlChange },
    /// A note on with a preceding HighResVelocity CC per CA-031
    HighResNoteOn { note: u8, velocity: u16 },
    /// A note off with a preceding HighResVelocity CC per CA-031
    HighResNoteOff { note: u8, velocity: u16 },
    /// The amount of pressure being applied to a given note, which is a signal some controllers
    /// after an initial `NoteOn`.
    /// Can act on multiple notes at a time, thus it is "polyphonic".
    PolyPressure {
        /// The note to apply this pressure signal to, 0-127
        note: u8,
        /// The amount of pressure to apply, 0-127
        pressure: u8,
    },
    /// Similar to `PolyPressure`, but only applies at the channel-level.
    ChannelPressure { pressure: u8 },
    /// Which "program", "patch" or "sound" to use when playing any preceding notes, 0-127.
    /// Use [`GMSoundSet`](crate::GMSoundSet) when targeting General MIDI
    ProgramChange { program: u8 },
    /// Apply a pitch bend to all sounding notes. 0-8191 represent negative bends,
    /// 8192 is no bend and8193-16383 are positive bends, with the standard bend rang
    /// being +/-2 semitones per GM2. See [`Parameter::PitchBendSensitivity`]
    PitchBend { bend: u16 },
}

impl ChannelVoiceMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            ChannelVoiceMsg::NoteOff { .. } => v.push(0x80),
            ChannelVoiceMsg::NoteOn { .. } => v.push(0x90),
            ChannelVoiceMsg::HighResNoteOff { .. } => v.push(0x80),
            ChannelVoiceMsg::HighResNoteOn { .. } => v.push(0x90),
            ChannelVoiceMsg::PolyPressure { .. } => v.push(0xA0),
            ChannelVoiceMsg::ControlChange { .. } => v.push(0xB0),
            ChannelVoiceMsg::ProgramChange { .. } => v.push(0xC0),
            ChannelVoiceMsg::ChannelPressure { .. } => v.push(0xD0),
            ChannelVoiceMsg::PitchBend { .. } => v.push(0xE0),
        }
        self.extend_midi_running(v);
    }

    // Can this message be extended by another?
    pub(crate) fn is_extensible(&self) -> bool {
        match self {
            Self::NoteOff { .. }
            | Self::NoteOn { .. }
            | Self::HighResNoteOff { .. }
            | Self::HighResNoteOn { .. } => true,
            Self::ControlChange {
                control: ControlChange::Parameter(_),
            } => true,
            Self::ControlChange { control } => control.is_lsb() || control.is_msb(),
            _ => false,
        }
    }

    // Can this message function as an extension to another?
    pub(crate) fn is_extension(&self) -> bool {
        match self {
            Self::ControlChange { control } => match control {
                ControlChange::HighResVelocity(_) => true,
                control => control.is_lsb() || control.is_msb(),
            },
            _ => false,
        }
    }

    pub(crate) fn maybe_extend(&self, other: &Self) -> Result<Self, ()> {
        match (self, other) {
            (
                Self::NoteOff { note, velocity },
                Self::ControlChange {
                    control: ControlChange::HighResVelocity(v),
                },
            ) => Ok(Self::HighResNoteOff {
                note: *note,
                velocity: u14_from_u7s(*velocity, *v),
            }),
            (
                Self::NoteOn { note, velocity },
                Self::ControlChange {
                    control: ControlChange::HighResVelocity(v),
                },
            ) => Ok(Self::HighResNoteOn {
                note: *note,
                velocity: u14_from_u7s(*velocity, *v),
            }),
            (
                Self::HighResNoteOff { note, velocity },
                Self::ControlChange {
                    control: ControlChange::HighResVelocity(v),
                },
            ) => Ok(Self::HighResNoteOff {
                note: *note,
                velocity: replace_u14_lsb(*velocity, *v),
            }),
            (
                Self::HighResNoteOn { note, velocity },
                Self::ControlChange {
                    control: ControlChange::HighResVelocity(v),
                },
            ) => Ok(Self::HighResNoteOn {
                note: *note,
                velocity: replace_u14_lsb(*velocity, *v),
            }),
            (Self::ControlChange { control: ctrl1 }, Self::ControlChange { control: ctrl2 }) => {
                match ctrl1.maybe_extend(ctrl2) {
                    Ok(control) => Ok(Self::ControlChange { control }),
                    Err(()) => Err(()),
                }
            }
            _ => Err(()),
        }
    }

    /// Out of necessity, pushes a Channel message after the note message for `HighResNoteOn/Off`
    pub(crate) fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match *self {
            ChannelVoiceMsg::NoteOff { note, velocity } => {
                v.push(to_u7(note));
                v.push(to_u7(velocity));
            }
            ChannelVoiceMsg::NoteOn { note, velocity } => {
                v.push(to_u7(note));
                v.push(to_u7(velocity));
            }
            ChannelVoiceMsg::HighResNoteOff { note, velocity } => {
                let [msb, lsb] = to_u14(velocity);
                push_u7(note, v);
                v.push(msb);
                v.push(0xB0);
                v.push(0x58);
                v.push(lsb);
            }
            ChannelVoiceMsg::HighResNoteOn { note, velocity } => {
                let [msb, lsb] = to_u14(velocity);
                push_u7(note, v);
                v.push(msb);
                v.push(0xB0);
                v.push(0x58);
                v.push(lsb);
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

    pub(crate) fn from_midi(m: &[u8]) -> Result<(Self, usize), ParseError> {
        let status = match m.first() {
            Some(b) => match b >> 4 {
                0x8 => Self::NoteOff {
                    note: 0,
                    velocity: 0,
                },
                0x9 => Self::NoteOn {
                    note: 0,
                    velocity: 0,
                },
                0xA => Self::PolyPressure {
                    note: 0,
                    pressure: 0,
                },
                0xB => Self::ControlChange {
                    control: ControlChange::BankSelect(0),
                },
                0xC => Self::ProgramChange { program: 0 },
                0xD => Self::ChannelPressure { pressure: 0 },
                0xE => Self::PitchBend { bend: 0 },
                _ => return Err(ParseError::Invalid("This shouldn't be possible")),
            },
            None => return Err(ParseError::UnexpectedEnd),
        };
        let (msg, len) = Self::from_midi_running(&m[1..], &status)?;
        Ok((msg, len + 1))
    }

    pub(crate) fn from_midi_running(m: &[u8], msg: &Self) -> Result<(Self, usize), ParseError> {
        match msg {
            Self::NoteOff { .. } => Ok((
                Self::NoteOff {
                    note: u7_from_midi(m)?,
                    velocity: u7_from_midi(&m[1..])?,
                },
                2,
            )),
            Self::NoteOn { .. } => Ok((
                Self::NoteOn {
                    note: u7_from_midi(m)?,
                    velocity: u7_from_midi(&m[1..])?,
                },
                2,
            )),
            Self::PolyPressure { .. } => Ok((
                Self::PolyPressure {
                    note: u7_from_midi(m)?,
                    pressure: u7_from_midi(&m[1..])?,
                },
                2,
            )),
            Self::ControlChange { .. } => Ok((
                Self::ControlChange {
                    control: ControlChange::from_midi(m)?,
                },
                2,
            )),
            Self::ProgramChange { .. } => Ok((
                Self::ProgramChange {
                    program: u7_from_midi(m)?,
                },
                1,
            )),
            Self::ChannelPressure { .. } => Ok((
                Self::ChannelPressure {
                    pressure: u7_from_midi(m)?,
                },
                1,
            )),
            Self::PitchBend { .. } => Ok((
                Self::PitchBend {
                    bend: u14_from_midi(m)?,
                },
                2,
            )),
            Self::HighResNoteOn { .. } | Self::HighResNoteOff { .. } => {
                // This shouldn't really be used as a running message, but if it is, the last
                // midi msg sent would have been a CC.
                Ok((
                    Self::ControlChange {
                        control: ControlChange::from_midi(m)?,
                    },
                    2,
                ))
            }
        }
    }
}

/// An enum that defines the MIDI numbers associated with Control Changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlNumber {
    BankSelect = 0,
    BankSelectLSB = 32,
    ModWheel = 1,
    ModWheelLSB = 33,
    Breath = 2,
    BreathLSB = 34,
    Foot = 4,
    FootLSB = 36,
    Portamento = 5,
    PortamentoLSB = 37,
    DataEntry = 6,
    DataEntryLSB = 38,
    Volume = 7,
    VolumeLSB = 39,
    Balance = 8,
    BalanceLSB = 40,
    Pan = 10,
    PanLSB = 42,
    Expression = 11,
    ExpressionLSB = 43,
    Effect1 = 12,
    Effect1LSB = 44,
    Effect2 = 13,
    Effect2LSB = 45,
    GeneralPurpose1 = 16,
    GeneralPurpose1LSB = 48,
    GeneralPurpose2 = 17,
    GeneralPurpose2LSB = 49,
    GeneralPurpose3 = 18,
    GeneralPurpose3LSB = 50,
    GeneralPurpose4 = 19,
    GeneralPurpose4LSB = 51,
    /// AKA Sustain
    Hold = 64,
    TogglePortamento = 65,
    Sostenuto = 66,
    SoftPedal = 67,
    ToggleLegato = 68,
    Hold2 = 69,
    /// AKA SoundVariation
    SoundControl1 = 70,
    /// AKA Timbre
    SoundControl2 = 71,
    /// AKA ReleaseTime
    SoundControl3 = 72,
    /// AKA AttackTime
    SoundControl4 = 73,
    /// AKA Brightness
    SoundControl5 = 74,
    /// AKA DecayTime
    SoundControl6 = 75,
    /// AKA VibratoRate
    SoundControl7 = 76,
    /// AKA VibratoDepth
    SoundControl8 = 77,
    /// AKA VibratoDelay
    SoundControl9 = 78,
    SoundControl10 = 79,
    GeneralPurpose5 = 80,
    GeneralPurpose6 = 81,
    GeneralPurpose7 = 82,
    GeneralPurpose8 = 83,
    PortamentoControl = 84,
    HighResVelocity = 88,
    /// AKA ReverbSendLevel
    Effects1Depth = 91,
    /// AKA TremoloDepth
    Effects2Depth = 92,
    /// AKA ChorusSendLevel
    Effects3Depth = 93,
    /// AKA CelesteDepth
    Effects4Depth = 94,
    /// AKA PhaserDepth
    Effects5Depth = 95,
    DataIncrement = 96,
    DataDecrement = 97,
    NonRegisteredParameterLSB = 98,
    NonRegisteredParameter = 99,
    RegisteredParameterLSB = 100,
    RegisteredParameter = 101,
}

/// Used by [`ChannelVoiceMsg::ControlChange`] to modify sounds.
/// Each control targets a particular [`ControlNumber`], the meaning of which is given by convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlChange {
    /// 0-16383
    BankSelect(u16),
    /// 0-16383
    ModWheel(u16),
    /// 0-16383
    Breath(u16),
    /// Control number may be any valid Midi CC Control number. May not be > 119.
    Undefined {
        control: u8,
        /// 0-127
        value: u8,
    },
    /// `control1` is associated with the MSB of the value, `control2` with the LSB. Neither `controls` may be > 119.
    UndefinedHighRes {
        control1: u8,
        control2: u8,
        /// 0-16383
        value: u16,
    },
    /// 0-16383
    Foot(u16),
    /// 0-16383
    Portamento(u16),
    /// 0-16383
    Volume(u16),
    /// 0-16383
    Balance(u16),
    /// 0-16383
    Pan(u16),
    /// 0-16383
    Expression(u16),
    /// 0-16383
    Effect1(u16),
    /// 0-16383
    Effect2(u16),
    /// 0-16383
    GeneralPurpose1(u16),
    /// 0-16383
    GeneralPurpose2(u16),
    /// 0-16383
    GeneralPurpose3(u16),
    /// 0-16383
    GeneralPurpose4(u16),
    /// 0-127
    GeneralPurpose5(u8),
    /// 0-127
    GeneralPurpose6(u8),
    /// 0-127
    GeneralPurpose7(u8),
    /// 0-127
    GeneralPurpose8(u8),
    /// 0-127
    Hold(u8),
    /// 0-127
    Hold2(u8),
    /// Turn portamento on or off
    TogglePortamento(bool),
    /// 0-127
    Sostenuto(u8),
    /// 0-127
    SoftPedal(u8),
    /// Turn legato on or off
    ToggleLegato(bool),
    /// Same as SoundControl1
    SoundVariation(u8),
    /// Same as SoundControl2
    Timbre(u8),
    /// Same as SoundControl3
    ReleaseTime(u8),
    /// Same as SoundControl4
    AttackTime(u8),
    /// Same as SoundControl5, and used as the MPE "third dimension" (usually Timbre) control
    /// (RP-021, RP-053)
    Brightness(u8),
    /// Same as SoundControl6 (RP-021)
    DecayTime(u8),
    /// Same as SoundControl7 (RP-021)
    VibratoRate(u8),
    /// Same as SoundControl8 (RP-021)
    VibratoDepth(u8),
    /// Same as SoundControl9 (RP-021)
    VibratoDelay(u8),
    /// 0-127
    SoundControl1(u8),
    /// 0-127
    SoundControl2(u8),
    /// 0-127
    SoundControl3(u8),
    /// 0-127
    SoundControl4(u8),
    /// 0-127
    SoundControl5(u8),
    /// 0-127
    SoundControl6(u8),
    /// 0-127
    SoundControl7(u8),
    /// 0-127
    SoundControl8(u8),
    /// 0-127
    SoundControl9(u8),
    /// 0-127
    SoundControl10(u8),
    /// Used as the LSB of the velocity for the next note on/off message, 0-127.
    /// Defined in CA-031
    HighResVelocity(u8),
    /// 0-127
    PortamentoControl(u8),
    /// 0-127
    Effects1Depth(u8),
    /// 0-127
    Effects2Depth(u8),
    /// 0-127
    Effects3Depth(u8),
    /// 0-127
    Effects4Depth(u8),
    /// 0-127
    Effects5Depth(u8),
    /// Same as Effects1Depth (RP-023)
    ReverbSendLevel(u8),
    /// Same as Effects2Depth
    TremoloDepth(u8),
    /// Same as Effects3Depth (RP-023)
    ChorusSendLevel(u8),
    /// Same as Effects4Depth
    CelesteDepth(u8),
    /// Same as Effects5Depth
    PhaserDepth(u8),
    /// Registered and Unregistered Parameters
    Parameter(Parameter),
    /// Set the value of the last-set Parameter. 0-16383
    DataEntry(u16),
    /// Set the MSB and LSB of the last-set parameter separately.
    DataEntry2(u8, u8),
    /// Increment the value of the last-set Parameter. 0-127
    DataIncrement(u8),
    /// Decrement the value of the last-set Parameter. 0-127
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

    fn is_msb(&self) -> bool {
        match self {
            Self::BankSelect(_)
            | Self::ModWheel(_)
            | Self::Breath(_)
            | Self::DataEntry(_)
            | Self::UndefinedHighRes { .. }
            | Self::Foot(_)
            | Self::Portamento(_)
            | Self::Volume(_)
            | Self::Balance(_)
            | Self::Pan(_)
            | Self::Expression(_)
            | Self::Effect1(_)
            | Self::Effect2(_)
            | Self::GeneralPurpose1(_)
            | Self::GeneralPurpose2(_)
            | Self::GeneralPurpose3(_)
            | Self::GeneralPurpose4(_) => true,
            Self::Undefined { control, .. }
                if control < &32 || control == &99 || control == &101 =>
            {
                true
            }
            _ => false,
        }
    }

    fn is_lsb(&self) -> bool {
        match self {
            Self::Undefined { control, .. }
                if control >= &32 && control < &64 || control == &98 || control == &100 =>
            {
                true
            }
            _ => false,
        }
    }

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
            ControlChange::GeneralPurpose1(x) => ControlChange::high_res_cc(v, 16, x),
            ControlChange::GeneralPurpose2(x) => ControlChange::high_res_cc(v, 17, x),
            ControlChange::GeneralPurpose3(x) => ControlChange::high_res_cc(v, 18, x),
            ControlChange::GeneralPurpose4(x) => ControlChange::high_res_cc(v, 19, x),
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
            ControlChange::SoundVariation(x) | ControlChange::SoundControl1(x) => {
                v.push(70);
                v.push(to_u7(x));
            }
            ControlChange::Timbre(x) | ControlChange::SoundControl2(x) => {
                v.push(71);
                v.push(to_u7(x));
            }
            ControlChange::ReleaseTime(x) | ControlChange::SoundControl3(x) => {
                v.push(72);
                v.push(to_u7(x));
            }
            ControlChange::AttackTime(x) | ControlChange::SoundControl4(x) => {
                v.push(73);
                v.push(to_u7(x));
            }
            ControlChange::Brightness(x) | ControlChange::SoundControl5(x) => {
                v.push(74);
                v.push(to_u7(x));
            }
            ControlChange::DecayTime(x) | ControlChange::SoundControl6(x) => {
                v.push(75);
                v.push(to_u7(x));
            }
            ControlChange::VibratoRate(x) | ControlChange::SoundControl7(x) => {
                v.push(76);
                v.push(to_u7(x));
            }
            ControlChange::VibratoDepth(x) | ControlChange::SoundControl8(x) => {
                v.push(77);
                v.push(to_u7(x));
            }
            ControlChange::VibratoDelay(x) | ControlChange::SoundControl9(x) => {
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
            ControlChange::HighResVelocity(x) => {
                v.push(88);
                v.push(to_u7(x));
            }
            ControlChange::Effects1Depth(x) | ControlChange::ReverbSendLevel(x) => {
                v.push(91);
                v.push(to_u7(x));
            }
            ControlChange::Effects2Depth(x) | ControlChange::TremoloDepth(x) => {
                v.push(92);
                v.push(to_u7(x));
            }
            ControlChange::Effects3Depth(x) | ControlChange::ChorusSendLevel(x) => {
                v.push(93);
                v.push(to_u7(x));
            }
            ControlChange::Effects4Depth(x) | ControlChange::CelesteDepth(x) => {
                v.push(94);
                v.push(to_u7(x));
            }
            ControlChange::Effects5Depth(x) | ControlChange::PhaserDepth(x) => {
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

    fn from_midi(m: &[u8]) -> Result<Self, ParseError> {
        if m.len() < 2 {
            return Err(crate::ParseError::UnexpectedEnd);
        }

        if m[0] > 119 {
            return Err(ParseError::Invalid("Tried to parse a control change message, but it looks like a channel mode message"));
        }

        let value = u8_from_u7(m[1])?;
        Ok(match m[0] {
            // 14 bit controls
            0 => Self::BankSelect((value as u16) << 7),
            1 => Self::ModWheel((value as u16) << 7),
            2 => Self::Breath((value as u16) << 7),
            4 => Self::Foot((value as u16) << 7),
            5 => Self::Portamento((value as u16) << 7),
            6 => Self::DataEntry((value as u16) << 7),
            7 => Self::Volume((value as u16) << 7),
            8 => Self::Balance((value as u16) << 7),
            10 => Self::Pan((value as u16) << 7),
            11 => Self::Expression((value as u16) << 7),
            12 => Self::Effect1((value as u16) << 7),
            13 => Self::Effect2((value as u16) << 7),
            16 => Self::GeneralPurpose1((value as u16) << 7),
            17 => Self::GeneralPurpose2((value as u16) << 7),
            18 => Self::GeneralPurpose3((value as u16) << 7),
            19 => Self::GeneralPurpose4((value as u16) << 7),
            // 7 bit controls
            64 => Self::Hold(value),
            65 => Self::TogglePortamento(bool_from_u7(m[1])?),
            66 => Self::Sostenuto(value),
            67 => Self::SoftPedal(value),
            68 => Self::ToggleLegato(bool_from_u7(m[1])?),
            69 => Self::Hold2(value),
            70 => Self::SoundControl1(value),
            71 => Self::SoundControl2(value),
            72 => Self::SoundControl3(value),
            73 => Self::SoundControl4(value),
            74 => Self::SoundControl5(value),
            75 => Self::SoundControl6(value),
            76 => Self::SoundControl7(value),
            77 => Self::SoundControl8(value),
            78 => Self::SoundControl9(value),
            79 => Self::SoundControl10(value),
            80 => Self::GeneralPurpose5(value),
            81 => Self::GeneralPurpose6(value),
            82 => Self::GeneralPurpose7(value),
            83 => Self::GeneralPurpose8(value),
            84 => Self::PortamentoControl(value),
            88 => Self::HighResVelocity(value),
            91 => Self::Effects1Depth(value),
            92 => Self::Effects2Depth(value),
            93 => Self::Effects3Depth(value),
            94 => Self::Effects4Depth(value),
            95 => Self::Effects5Depth(value),
            96 => Self::DataIncrement(value),
            97 => Self::DataDecrement(value),
            // Undefined controls (including parameters)
            3 | 9 | 14 | 15 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 => {
                Self::UndefinedHighRes {
                    control1: m[0],
                    control2: m[0] + 32,
                    value: (value as u16) << 7,
                }
            }
            control => Self::Undefined { control, value },
        })
    }

    fn maybe_extend(&self, other: &Self) -> Result<Self, ()> {
        match (self, other) {
            (Self::BankSelect(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::BankSelect(msb))
                if *control == ControlNumber::BankSelectLSB as u8 =>
            {
                Ok(Self::BankSelect(replace_u14_lsb(*msb, *value)))
            }
            (Self::ModWheel(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::ModWheel(msb))
                if *control == ControlNumber::ModWheelLSB as u8 =>
            {
                Ok(Self::ModWheel(replace_u14_lsb(*msb, *value)))
            }
            (Self::Breath(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Breath(msb))
                if *control == ControlNumber::BreathLSB as u8 =>
            {
                Ok(Self::Breath(replace_u14_lsb(*msb, *value)))
            }
            (Self::Foot(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Foot(msb))
                if *control == ControlNumber::FootLSB as u8 =>
            {
                Ok(Self::Foot(replace_u14_lsb(*msb, *value)))
            }
            (Self::Portamento(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Portamento(msb))
                if *control == ControlNumber::PortamentoLSB as u8 =>
            {
                Ok(Self::Portamento(replace_u14_lsb(*msb, *value)))
            }
            (Self::DataEntry(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::DataEntry(msb))
                if *control == ControlNumber::DataEntryLSB as u8 =>
            {
                Ok(Self::DataEntry(replace_u14_lsb(*msb, *value)))
            }
            (Self::Volume(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Volume(msb))
                if *control == ControlNumber::VolumeLSB as u8 =>
            {
                Ok(Self::Volume(replace_u14_lsb(*msb, *value)))
            }
            (Self::Balance(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Balance(msb))
                if *control == ControlNumber::BalanceLSB as u8 =>
            {
                Ok(Self::Balance(replace_u14_lsb(*msb, *value)))
            }
            (Self::Pan(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Pan(msb))
                if *control == ControlNumber::PanLSB as u8 =>
            {
                Ok(Self::Pan(replace_u14_lsb(*msb, *value)))
            }
            (Self::Expression(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Expression(msb))
                if *control == ControlNumber::ExpressionLSB as u8 =>
            {
                Ok(Self::Expression(replace_u14_lsb(*msb, *value)))
            }
            (Self::Effect1(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Effect1(msb))
                if *control == ControlNumber::Effect1LSB as u8 =>
            {
                Ok(Self::Effect1(replace_u14_lsb(*msb, *value)))
            }
            (Self::Effect2(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Effect2(msb))
                if *control == ControlNumber::Effect2LSB as u8 =>
            {
                Ok(Self::Effect2(replace_u14_lsb(*msb, *value)))
            }
            (Self::GeneralPurpose1(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::GeneralPurpose1(msb))
                if *control == ControlNumber::GeneralPurpose1LSB as u8 =>
            {
                Ok(Self::GeneralPurpose1(replace_u14_lsb(*msb, *value)))
            }
            (Self::GeneralPurpose2(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::GeneralPurpose2(msb))
                if *control == ControlNumber::GeneralPurpose2LSB as u8 =>
            {
                Ok(Self::GeneralPurpose2(replace_u14_lsb(*msb, *value)))
            }
            (Self::GeneralPurpose3(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::GeneralPurpose3(msb))
                if *control == ControlNumber::GeneralPurpose3LSB as u8 =>
            {
                Ok(Self::GeneralPurpose3(replace_u14_lsb(*msb, *value)))
            }
            (Self::GeneralPurpose4(msb), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::GeneralPurpose4(msb))
                if *control == ControlNumber::GeneralPurpose4LSB as u8 =>
            {
                Ok(Self::GeneralPurpose4(replace_u14_lsb(*msb, *value)))
            }
            (
                Self::UndefinedHighRes {
                    control1,
                    control2,
                    value: msb,
                },
                Self::Undefined { control, value },
            )
            | (
                Self::Undefined { control, value },
                Self::UndefinedHighRes {
                    control1,
                    control2,
                    value: msb,
                },
            ) if control == control2 => Ok(Self::UndefinedHighRes {
                control1: *control1,
                control2: *control2,
                value: replace_u14_lsb(*msb, *value),
            }),
            // Parameters
            (
                Self::Undefined {
                    control: ctrl1,
                    value: val1,
                },
                Self::Undefined {
                    control: ctrl2,
                    value: val2,
                },
            ) => {
                let ((ctrl_lsb, ctrl_msb), (val_lsb, val_msb)) = if ctrl1 < ctrl2 {
                    ((*ctrl1, *ctrl2), (*val1, *val2))
                } else {
                    ((*ctrl2, *ctrl1), (*val2, *val1))
                };

                if ctrl_lsb == ControlNumber::NonRegisteredParameterLSB as u8
                    && ctrl_msb == ControlNumber::NonRegisteredParameter as u8
                {
                    Ok(Self::Parameter(Parameter::Unregistered(u14_from_u7s(
                        val_msb, val_lsb,
                    ))))
                } else if ctrl_lsb == ControlNumber::RegisteredParameterLSB as u8
                    && ctrl_msb == ControlNumber::RegisteredParameter as u8
                {
                    Ok(Self::Parameter(Parameter::maybe_extend_cc(
                        val_msb, val_lsb,
                    )?))
                } else {
                    Err(())
                }
            }
            (Self::Parameter(param), Self::Undefined { control, value })
            | (Self::Undefined { control, value }, Self::Parameter(param))
                if *control == ControlNumber::DataEntryLSB as u8 =>
            {
                Ok(Self::Parameter(param.maybe_extend(None, Some(*value))?))
            }

            (Self::Parameter(param), Self::DataEntry(value))
            | (Self::DataEntry(value), Self::Parameter(param)) => {
                Ok(Self::Parameter(param.maybe_extend(Some(*value), None)?))
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Used by [`ControlChange::Parameter`]. "Entry" Parameters can be used to set the given parameters:
/// they will first select that parameter, then send a [`ControlChange::DataEntry`] with the given value.
pub enum Parameter {
    /// A whole bunch of parameters defined by the given number 0-16383, that can be used for whatever.
    Unregistered(u16),
    /// A registered parameter that does nothing.
    /// Defined in GM2.
    Null,
    /// The pitch bend sensitivity in semitones (0-127) and the sensitivity in cents (0-100),
    /// respectively. For example, a value (1, 0) means +/- one semitone (a total range of two
    /// semitones)
    PitchBendSensitivity,
    PitchBendSensitivityEntry(u8, u8),
    /// A value from -8192-8191, representing the fractional cents to shift away from A440
    /// in 1/8192ths of a cent.
    FineTuning,
    FineTuningEntry(i16),
    /// A value from -64-63, the number of semitones to shift away from A44.
    CoarseTuning,
    CoarseTuningEntry(i8),
    /// Which "Tuning Program" to select from: 0-127.
    ///
    /// Defined in the MIDI Tuning Standard (Updated Specification)
    TuningProgramSelect,
    TuningProgramSelectEntry(u8),
    /// Which "Tuning Bank" to select from: 0-127.
    ///
    /// Defined in the MIDI Tuning Standard (Updated Specification)
    TuningBankSelect,
    TuningBankSelectEntry(u8),
    /// The amount of "modulation depth" your mod wheel should apply: 0-16383.
    ///
    /// Defined in CA 26. GM2 defines what this range might mean
    ModulationDepthRange,
    ModulationDepthRangeEntry(u16),
    /// Only valid when sent to channel 1 or channel 16, the former indicating that this
    /// is configuring the number of "lower zone" channels and the latter referring to the
    /// "upper zone". A value between 0 (zone is not configured to be MPE) and 16 (zone has
    /// 16 channels in it). There can be no more than lower zone channels + upper zone channels
    /// active at a given time.
    ///
    /// Defined in RP-053: MIDI Polyphonic Expression
    PolyphonicExpression,
    PolyphonicExpressionEntry(u8),
    /// A value 0-16383 representing -180.00-179.98 degrees.
    ///
    /// Defined in RP-049
    AzimuthAngle3DSound,
    AzimuthAngle3DSoundEntry(u16),
    /// A value 0-16383 representing -180.00-179.98 degrees.
    ///
    /// Defined in RP-049
    ElevationAngle3DSound,
    ElevationAngle3DSoundEntry(u16),
    /// A value 1-16383 representing -163.82-0 dB of gain.
    ///
    /// 0 indicates "negative infinity".
    ///
    /// Defined in RP-049
    Gain3DSound,
    Gain3DSoundEntry(u16),
    /// A value 0-16383 representing a ratio between -0.000061-1.0.
    ///
    /// Defined in RP-049
    DistanceRatio3DSound,
    DistanceRatio3DSoundEntry(u16),
    /// A value 0-16383 representing between 0 and 1000 distance units.
    /// Defined in RP-049
    MaxiumumDistance3DSound,
    MaxiumumDistance3DSoundEntry(u16),
    /// A value 0-16383 representing -163.83-0 dB of gain
    /// Defined in RP-049
    GainAtMaxiumumDistance3DSound,
    GainAtMaxiumumDistance3DSoundEntry(u16),
    /// A value 0-16383 representing a ratio between -0.000061-1.0
    /// Defined in RP-049
    ReferenceDistanceRatio3DSound,
    ReferenceDistanceRatio3DSoundEntry(u16),
    /// A value 0-16383 representing -180.00-179.98 degrees
    /// Defined in RP-049
    PanSpreadAngle3DSound,
    PanSpreadAngle3DSoundEntry(u16),
    /// A value 0-16383 representing -180.00-179.98 degrees
    /// Defined in RP-049
    RollAngle3DSound,
    RollAngle3DSoundEntry(u16),
}

impl Parameter {
    fn extend_midi_running(&self, v: &mut Vec<u8>) {
        match self {
            Self::Null => {
                v.push(100);
                v.push(0x7F);
                v.push(101);
                v.push(0x7F);
            }
            Self::PitchBendSensitivity => {
                v.push(100);
                v.push(0);
                v.push(101);
                v.push(0);
            }
            Self::PitchBendSensitivityEntry(c, f) => {
                Self::PitchBendSensitivity.extend_midi_running(v);
                // Data entry:
                v.push(6);
                v.push(*c);
                v.push(6 + 32);
                v.push((*f).min(100));
            }
            Self::FineTuning => {
                v.push(100);
                v.push(1);
                v.push(101);
                v.push(0);
            }
            Self::FineTuningEntry(x) => {
                Self::FineTuning.extend_midi_running(v);
                // Data entry:
                let [msb, lsb] = i_to_u14(*x);
                v.push(6);
                v.push(msb);
                v.push(6 + 32);
                v.push(lsb);
            }
            Self::CoarseTuning => {
                v.push(100);
                v.push(2);
                v.push(101);
                v.push(0);
            }
            Self::CoarseTuningEntry(x) => {
                Self::CoarseTuning.extend_midi_running(v);
                // Data entry:
                let msb = i_to_u7(*x);
                v.push(6);
                v.push(msb);
                v.push(6 + 32);
                v.push(0);
            }
            Self::TuningProgramSelect => {
                v.push(100);
                v.push(3);
                v.push(101);
                v.push(0);
            }
            Self::TuningProgramSelectEntry(x) => {
                Self::TuningProgramSelect.extend_midi_running(v);
                // Data entry (MSB only)
                v.push(6);
                v.push(*x);
            }
            Self::TuningBankSelect => {
                v.push(100);
                v.push(4);
                v.push(101);
                v.push(0);
            }
            Self::TuningBankSelectEntry(x) => {
                Self::TuningBankSelect.extend_midi_running(v);
                // Data entry (MSB only)
                v.push(6);
                v.push(*x);
            }
            Self::ModulationDepthRange => {
                v.push(100);
                v.push(5);
                v.push(101);
                v.push(0);
            }
            Self::ModulationDepthRangeEntry(x) => {
                Self::ModulationDepthRange.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::PolyphonicExpression => {
                v.push(100);
                v.push(6);
                v.push(101);
                v.push(0);
            }
            Self::PolyphonicExpressionEntry(x) => {
                Self::PolyphonicExpression.extend_midi_running(v);
                // Data entry (MSB only)
                v.push(6);
                v.push((*x).min(16));
            }
            Self::AzimuthAngle3DSound => {
                v.push(100);
                v.push(0);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::AzimuthAngle3DSoundEntry(x) => {
                Self::AzimuthAngle3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::ElevationAngle3DSound => {
                v.push(100);
                v.push(1);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::ElevationAngle3DSoundEntry(x) => {
                Self::ElevationAngle3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::Gain3DSound => {
                v.push(100);
                v.push(2);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::Gain3DSoundEntry(x) => {
                Self::Gain3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::DistanceRatio3DSound => {
                v.push(100);
                v.push(3);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::DistanceRatio3DSoundEntry(x) => {
                Self::DistanceRatio3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::MaxiumumDistance3DSound => {
                v.push(100);
                v.push(4);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::MaxiumumDistance3DSoundEntry(x) => {
                Self::MaxiumumDistance3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::GainAtMaxiumumDistance3DSound => {
                v.push(100);
                v.push(5);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::GainAtMaxiumumDistance3DSoundEntry(x) => {
                Self::GainAtMaxiumumDistance3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::ReferenceDistanceRatio3DSound => {
                v.push(100);
                v.push(6);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::ReferenceDistanceRatio3DSoundEntry(x) => {
                Self::ReferenceDistanceRatio3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::PanSpreadAngle3DSound => {
                v.push(100);
                v.push(7);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::PanSpreadAngle3DSoundEntry(x) => {
                Self::PanSpreadAngle3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::RollAngle3DSound => {
                v.push(100);
                v.push(8);
                v.push(101);
                v.push(61); // 3D Sound
            }
            Self::RollAngle3DSoundEntry(x) => {
                Self::RollAngle3DSound.extend_midi_running(v);
                // Data entry
                ControlChange::high_res_cc(v, 6, *x);
            }
            Self::Unregistered(x) => {
                let [msb, lsb] = to_u14(*x);
                v.push(98);
                v.push(lsb);
                v.push(99);
                v.push(msb);
            }
        }
    }

    fn maybe_extend_cc(msb: u8, lsb: u8) -> Result<Self, ()> {
        match (msb, lsb) {
            (0x7F, 0x7F) => Ok(Self::Null),
            (0, 0) => Ok(Self::PitchBendSensitivity),
            (0, 1) => Ok(Self::FineTuning),
            (0, 2) => Ok(Self::CoarseTuning),
            (0, 3) => Ok(Self::TuningProgramSelect),
            (0, 4) => Ok(Self::TuningBankSelect),
            (0, 5) => Ok(Self::ModulationDepthRange),
            (0, 6) => Ok(Self::PolyphonicExpression),
            (61, 0) => Ok(Self::AzimuthAngle3DSound),
            (61, 1) => Ok(Self::ElevationAngle3DSound),
            (61, 2) => Ok(Self::Gain3DSound),
            (61, 3) => Ok(Self::DistanceRatio3DSound),
            (61, 4) => Ok(Self::MaxiumumDistance3DSound),
            (61, 5) => Ok(Self::GainAtMaxiumumDistance3DSound),
            (61, 6) => Ok(Self::ReferenceDistanceRatio3DSound),
            (61, 7) => Ok(Self::PanSpreadAngle3DSound),
            (61, 8) => Ok(Self::RollAngle3DSound),
            _ => Err(()),
        }
    }

    fn maybe_extend(&self, msb: Option<u16>, lsb: Option<u8>) -> Result<Self, ()> {
        match self {
            Self::PitchBendSensitivity => Ok(Self::PitchBendSensitivityEntry(
                msb.map_or(0, |v| (v >> 7) as u8),
                lsb.unwrap_or(0),
            )),
            Self::PitchBendSensitivityEntry(v1, v2) => Ok(Self::PitchBendSensitivityEntry(
                msb.map_or(*v1, |v| v as u8),
                lsb.unwrap_or(*v2),
            )),
            Self::FineTuning => Ok(Self::FineTuningEntry(i14_from_u7s(
                msb.map_or(0, |v| (v >> 7) as u8),
                lsb.unwrap_or(0),
            ))),
            Self::FineTuningEntry(v) => Ok(Self::FineTuningEntry(i14_from_u7s(
                msb.map_or(i_to_u14(*v)[0], |v| v as u8),
                lsb.unwrap_or(i_to_u14(*v)[1]),
            ))),
            Self::CoarseTuning => Ok(Self::CoarseTuningEntry(msb.map_or(0, |v| u7_to_i(v as u8)))),
            Self::CoarseTuningEntry(v) => Ok(Self::CoarseTuningEntry(
                msb.map_or(*v, |v| u7_to_i(v as u8)),
            )),
            Self::TuningProgramSelect => {
                Ok(Self::TuningProgramSelectEntry(msb.map_or(0, |v| v as u8)))
            }
            Self::TuningProgramSelectEntry(v) => {
                Ok(Self::TuningProgramSelectEntry(msb.map_or(*v, |v| v as u8)))
            }
            Self::TuningBankSelect => Ok(Self::TuningBankSelectEntry(msb.map_or(0, |v| v as u8))),
            Self::TuningBankSelectEntry(v) => {
                Ok(Self::TuningBankSelectEntry(msb.map_or(*v, |v| v as u8)))
            }
            Self::ModulationDepthRange => Ok(Self::ModulationDepthRangeEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::ModulationDepthRangeEntry(v) => Ok(Self::ModulationDepthRangeEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::PolyphonicExpression => {
                Ok(Self::PolyphonicExpressionEntry(msb.map_or(0, |v| v as u8)))
            }
            Self::PolyphonicExpressionEntry(v) => {
                Ok(Self::PolyphonicExpressionEntry(msb.map_or(*v, |v| v as u8)))
            }
            Self::AzimuthAngle3DSound => Ok(Self::AzimuthAngle3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::AzimuthAngle3DSoundEntry(v) => Ok(Self::AzimuthAngle3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::ElevationAngle3DSound => Ok(Self::ElevationAngle3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::ElevationAngle3DSoundEntry(v) => Ok(Self::ElevationAngle3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::Gain3DSound => Ok(Self::Gain3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::Gain3DSoundEntry(v) => Ok(Self::Gain3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(*v),
                lsb.unwrap_or((*v as u8) & 0b01111111),
            ))),
            Self::DistanceRatio3DSound => Ok(Self::DistanceRatio3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::DistanceRatio3DSoundEntry(v) => Ok(Self::DistanceRatio3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::MaxiumumDistance3DSound => Ok(Self::MaxiumumDistance3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(0), lsb.unwrap_or(0)),
            )),
            Self::MaxiumumDistance3DSoundEntry(v) => Ok(Self::MaxiumumDistance3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::GainAtMaxiumumDistance3DSound => Ok(Self::GainAtMaxiumumDistance3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(0), lsb.unwrap_or(0)),
            )),
            Self::GainAtMaxiumumDistance3DSoundEntry(v) => {
                Ok(Self::GainAtMaxiumumDistance3DSoundEntry(replace_u14_lsb(
                    msb.unwrap_or(*v),
                    lsb.unwrap_or((*v as u8) & 0b01111111),
                )))
            }
            Self::ReferenceDistanceRatio3DSound => Ok(Self::ReferenceDistanceRatio3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(0), lsb.unwrap_or(0)),
            )),
            Self::ReferenceDistanceRatio3DSoundEntry(v) => {
                Ok(Self::ReferenceDistanceRatio3DSoundEntry(replace_u14_lsb(
                    msb.unwrap_or(*v),
                    lsb.unwrap_or((*v as u8) & 0b01111111),
                )))
            }
            Self::PanSpreadAngle3DSound => Ok(Self::PanSpreadAngle3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::PanSpreadAngle3DSoundEntry(v) => Ok(Self::PanSpreadAngle3DSoundEntry(
                replace_u14_lsb(msb.unwrap_or(*v), lsb.unwrap_or((*v as u8) & 0b01111111)),
            )),
            Self::RollAngle3DSound => Ok(Self::RollAngle3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(0),
                lsb.unwrap_or(0),
            ))),
            Self::RollAngle3DSoundEntry(v) => Ok(Self::RollAngle3DSoundEntry(replace_u14_lsb(
                msb.unwrap_or(*v),
                lsb.unwrap_or((*v as u8) & 0b01111111),
            ))),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

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

    #[test]
    fn deserialize_channel_voice_msg() {
        let mut ctx = ReceiverContext::new();

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch1,
                msg: ChannelVoiceMsg::NoteOn {
                    note: 0x7f,
                    velocity: 0x7f,
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch10,
                msg: ChannelVoiceMsg::PitchBend { bend: 1000 },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Volume(1000),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch4,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Undefined {
                        control: 85,
                        value: 77,
                    },
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::UndefinedHighRes {
                        control1: 3,
                        control2: 35,
                        value: 1000,
                    },
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch3,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::TogglePortamento(true),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::FineTuning),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::Unregistered(1000)),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch3,
                msg: ChannelVoiceMsg::HighResNoteOn {
                    note: 77,
                    velocity: 1000,
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::FineTuningEntry(-30)),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::Gain3DSoundEntry(1001)),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::Parameter(Parameter::PitchBendSensitivityEntry(4, 78)),
                },
            },
            &mut ctx,
        );

        test_serialization(
            MidiMsg::ChannelVoice {
                channel: Channel::Ch2,
                msg: ChannelVoiceMsg::ControlChange {
                    control: ControlChange::GeneralPurpose1(50),
                },
            },
            &mut ctx,
        );
    }
}
