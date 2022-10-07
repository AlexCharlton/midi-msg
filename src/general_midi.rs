use strum::{EnumIter, Display, FromRepr, EnumString};

/// Used to turn General MIDI level 1 or 2 on, or turn them off.
///
/// Used in [`UniversalNonRealTimeMsg::GeneralMidi`](crate::UniversalNonRealTimeMsg::GeneralMidi)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneralMidi {
    GM1 = 1,
    GM2 = 3,
    Off = 2,
}

/// The instrument that should be played when applying a [`ChannelVoiceMsg::ProgramChange`](crate::ChannelVoiceMsg::ProgramChange).
///
/// Use `GMSoundSet::Sound as u8` to use as the program number. For example:
///
/// ```
/// # use midi_msg::*;
/// MidiMsg::ChannelVoice {
///     channel: Channel::Ch1,
///     msg: ChannelVoiceMsg::ProgramChange {
///         program: GMSoundSet::Vibraphone as u8
///     }
/// };
/// ```
///
/// Should not be used when targeting channel 10.
///
/// As defined in General MIDI System Level 1 (MMA0007 / RP003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, FromRepr, EnumString)]
#[repr(u8)]
pub enum GMSoundSet {
    AcousticGrandPiano,
    BrightAcousticPiano,
    ElectricGrandPiano,
    HonkytonkPiano,
    ElectricPiano1,
    ElectricPiano2,
    Harpsichord,
    Clavi,
    Celesta,
    Glockenspiel,
    MusicBox,
    Vibraphone,
    Marimba,
    Xylophone,
    TubularBells,
    Dulcimer,
    DrawbarOrgan,
    PercussiveOrgan,
    RockOrgan,
    ChurchOrgan,
    ReedOrgan,
    Accordion,
    Harmonica,
    TangoAccordion,
    AcousticGuitarNylon,
    AcousticGuitarSteel,
    ElectricGuitarJazz,
    ElectricGuitarClean,
    ElectricGuitarMuted,
    OverdrivenGuitar,
    DistortionGuitar,
    GuitarHarmonics,
    AcousticBass,
    ElectricBassFinger,
    ElectricBassPick,
    FretlessBass,
    SlapBass1,
    SlapBass2,
    SynthBass1,
    SynthBass2,
    Violin,
    Viola,
    Cello,
    Contrabass,
    TremoloStrings,
    PizzicatoStrings,
    OrchestralHarp,
    Timpani,
    StringEnsemble1,
    StringEnsemble2,
    SynthStrings1,
    SynthStrings2,
    ChoirAahs,
    VoiceOohs,
    SynthVoice,
    OrchestraHit,
    Trumpet,
    Trombone,
    Tuba,
    MutedTrumpet,
    FrenchHorn,
    BrassSection,
    SynthBrass1,
    SynthBrass2,
    SopranoSax,
    AltoSax,
    TenorSax,
    BaritoneSax,
    Oboe,
    EnglishHorn,
    Bassoon,
    Clarinet,
    Piccolo,
    Flute,
    Recorder,
    PanFlute,
    BlownBottle,
    Shakuhachi,
    Whistle,
    Ocarina,
    Lead1,
    Lead2,
    Lead3,
    Lead4,
    Lead5,
    Lead6,
    Lead7,
    Lead8,
    Pad1,
    Pad2,
    Pad3,
    Pad4,
    Pad5,
    Pad6,
    Pad7,
    Pad8,
    FX1,
    FX2,
    FX3,
    FX4,
    FX5,
    FX6,
    FX7,
    FX8,
    Sitar,
    Banjo,
    Shamisen,
    Koto,
    Kalimba,
    Bagpipe,
    Fiddle,
    Shanai,
    TinkleBell,
    Agogo,
    SteelDrums,
    Woodblock,
    TaikoDrum,
    MelodicTom,
    SynthDrum,
    ReverseCymbal,
    GuitarFretNoise,
    BreathNoise,
    Seashore,
    BirdTweet,
    TelephoneRing,
    Helicopter,
    Applause,
    Gunshot,
}

/// The General MIDI percussion sound to play for a given note number when targeting
/// Channel 10.
///
/// For example:
///
/// ```
/// # use midi_msg::*;
/// MidiMsg::ChannelVoice {
///     channel: Channel::Ch10,
///     msg: ChannelVoiceMsg::NoteOn {
///         note: GMPercussionMap::Vibraslap as u8,
///         velocity: 127
///     }
/// };
/// ```
///
/// As defined in General MIDI System Level 1 (MMA0007 / RP003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, FromRepr)]
#[repr(u8)]
pub enum GMPercussionMap {
    AcousticBassDrum = 35,
    BassDrum1,
    SideStick,
    AcousticSnare,
    HandClap,
    ElectricSnare,
    LowFloorTom,
    ClosedHiHat,
    HighFloorTom,
    PedalHiHat,
    LowTom,
    OpenHiHat,
    LowMidTom,
    HiMidTom,
    CrashCymbal1,
    HighTom,
    RideCymbal1,
    ChineseCymbal,
    RideBell,
    Tambourine,
    SplashCymbal,
    Cowbell,
    CrashCymbal2,
    Vibraslap,
    RideCymbal2,
    HiBongo,
    LowBongo,
    MuteHiConga,
    OpenHiConga,
    LowConga,
    HighTimbale,
    LowTimbale,
    HighAgogo,
    LowAgogo,
    Cabasa,
    Maracas,
    ShortWhistle,
    LongWhistle,
    ShortGuiro,
    LongGuiro,
    Claves,
    HiWoodBlock,
    LowWoodBlock,
    MuteCuica,
    OpenCuica,
    MuteTriangle,
    OpenTriangle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;
    use std::str::FromStr;

    #[test]
    fn gm_iter() {
        for (i, inst) in GMSoundSet::iter().enumerate() {
            //println!("{:?} {}",inst, inst as u8);
            assert_eq!(inst as u8, i as u8);
        }
    }

    #[test]
    fn gm_from_string() {
        assert_eq!(GMSoundSet::TenorSax, GMSoundSet::from_str("TenorSax").unwrap());
    }

    #[test]
    fn gm_display() {
        assert_eq!("TenorSax", format!("{}", GMSoundSet::TenorSax));
    }

    #[test]
    fn gm_tostring() {
        assert_eq!("TenorSax", GMSoundSet::TenorSax.to_string());
    }

    #[test]
    fn gm_from_u8() {
        let inst: GMSoundSet = GMSoundSet::from_repr(0).unwrap();
        assert_eq!(GMSoundSet::AcousticGrandPiano, inst);

        let inst: GMSoundSet = GMSoundSet::from_repr(127).unwrap();
        assert_eq!(GMSoundSet::Gunshot, inst);
    }

    #[test]
    fn gm_as_u8() {
        assert_eq!(0, GMSoundSet::AcousticGrandPiano as u8);

        assert_eq!(127, GMSoundSet::Gunshot as u8);
    }

    #[test]
    fn percussion_iter() {
        for (i, perc) in GMPercussionMap::iter().enumerate() {
            //println!("{:?} {}",inst, inst as u8);
            assert_eq!(perc as u8, (i + 35) as u8);
        }
    }
}