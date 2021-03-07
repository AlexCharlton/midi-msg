use super::util::*;

/// Channel-level messages that act on a voice. For instance, turning notes on off,
/// or modifying sounding notes. Used in [`MidiMsg`](crate::MidiMsg).
#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), crate::ParseError> {
        // let (msg, len) = ChannelModeMsg::from_midi_running(m[1..], TODO)?;
        // Ok((msg, len + 1))
        Err(crate::ParseError::Invalid("TODO".to_string()))
    }

    pub(crate) fn from_midi_running(
        _m: &[u8],
        msg: &Self,
    ) -> Result<(Self, usize), crate::ParseError> {
        Err(crate::ParseError::Invalid("TODO".to_string()))
    }
}

/// An enum that defines the MIDI numbers associated with Control Changes.
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
}

#[cfg(test)]
mod tests {
    use crate::*;

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
        let mut ctx = ReceiverContext::default();

        // test_serialization(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff,
        //         },
        //     },
        //     &mut ctx,
        // );
    }
}
