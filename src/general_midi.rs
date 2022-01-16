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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GMSoundSet {
    AcousticGrandPiano = 0,
    BrightAcousticPiano = 1,
    ElectricGrandPiano = 2,
    HonkytonkPiano = 3,
    ElectricPiano1 = 4,
    ElectricPiano2 = 5,
    Harpsichord = 6,
    Clavi = 7,
    Celesta = 8,
    Glockenspiel = 9,
    MusicBox = 10,
    Vibraphone = 11,
    Marimba = 12,
    Xylophone = 13,
    TubularBells = 14,
    Dulcimer = 15,
    DrawbarOrgan = 16,
    PercussiveOrgan = 17,
    RockOrgan = 18,
    ChurchOrgan = 19,
    ReedOrgan = 20,
    Accordion = 21,
    Harmonica = 22,
    TangoAccordion = 23,
    AcousticGuitarNylon = 24,
    AcousticGuitarSteel = 25,
    ElectricGuitarJazz = 26,
    ElectricGuitarClean = 27,
    ElectricGuitarMuted = 28,
    OverdrivenGuitar = 29,
    DistortionGuitar = 30,
    GuitarHarmonics = 31,
    AcousticBass = 32,
    ElectricBassFinger = 33,
    ElectricBassPick = 34,
    FretlessBass = 35,
    SlapBass1 = 36,
    SlapBass2 = 37,
    SynthBass1 = 38,
    SynthBass2 = 39,
    Violin = 40,
    Viola = 41,
    Cello = 42,
    Contrabass = 43,
    TremoloStrings = 44,
    PizzicatoStrings = 45,
    OrchestralHarp = 46,
    Timpani = 47,
    StringEnsemble1 = 48,
    StringEnsemble2 = 49,
    SynthStrings1 = 50,
    SynthStrings2 = 51,
    ChoirAahs = 52,
    VoiceOohs = 53,
    SynthVoice = 54,
    OrchestraHit = 55,
    Trumpet = 56,
    Trombone = 57,
    Tuba = 58,
    MutedTrumpet = 59,
    FrenchHorn = 60,
    BrassSection = 61,
    SynthBrass1 = 62,
    SynthBrass2 = 63,
    SopranoSax = 64,
    AltoSax = 65,
    TenorSax = 66,
    BaritoneSax = 67,
    Oboe = 68,
    EnglishHorn = 69,
    Bassoon = 70,
    Clarinet = 71,
    Piccolo = 72,
    Flute = 73,
    Recorder = 74,
    PanFlute = 75,
    BlownBottle = 76,
    Shakuhachi = 77,
    Whistle = 78,
    Ocarina = 79,
    Lead1 = 80,
    Lead2 = 81,
    Lead3 = 82,
    Lead4 = 83,
    Lead5 = 84,
    Lead6 = 85,
    Lead7 = 86,
    Lead8 = 87,
    Pad1 = 88,
    Pad2 = 89,
    Pad3 = 90,
    Pad4 = 91,
    Pad5 = 92,
    Pad6 = 93,
    Pad7 = 94,
    Pad8 = 95,
    FX1 = 96,
    FX2 = 97,
    FX3 = 98,
    FX4 = 99,
    FX5 = 100,
    FX6 = 101,
    FX7 = 102,
    FX8 = 103,
    Sitar = 104,
    Banjo = 105,
    Shamisen = 106,
    Koto = 107,
    Kalimba = 108,
    Bagpipe = 109,
    Fiddle = 110,
    Shanai = 111,
    TinkleBell = 112,
    Agogo = 113,
    SteelDrums = 114,
    Woodblock = 115,
    TaikoDrum = 116,
    MelodicTom = 117,
    SynthDrum = 118,
    ReverseCymbal = 119,
    GuitarFretNoise = 120,
    BreathNoise = 121,
    Seashore = 122,
    BirdTweet = 123,
    TelephoneRing = 124,
    Helicopter = 125,
    Applause = 126,
    Gunshot = 127,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GMPercussionMap {
    AcousticBassDrum = 35,
    RideCymbal1 = 51,
    HighAgogo = 67,
    BassDrum1 = 36,
    ChineseCymbal = 52,
    LowAgogo = 68,
    SideStick = 37,
    RideBell = 53,
    Cabasa = 69,
    AcousticSnare = 38,
    Tambourine = 54,
    Maracas = 70,
    HandClap = 39,
    SplashCymbal = 55,
    ShortWhistle = 71,
    ElectricSnare = 40,
    Cowbell = 56,
    LongWhistle = 72,
    LowFloorTom = 41,
    CrashCymbal2 = 57,
    ShortGuiro = 73,
    ClosedHiHat = 42,
    Vibraslap = 58,
    LongGuiro = 74,
    HighFloorTom = 43,
    RideCymbal2 = 59,
    Claves = 75,
    PedalHiHat = 44,
    HiBongo = 60,
    HiWoodBlock = 76,
    LowTom = 45,
    LowBongo = 61,
    LowWoodBlock = 77,
    OpenHiHat = 46,
    MuteHiConga = 62,
    MuteCuica = 78,
    LowMidTom = 47,
    OpenHiConga = 63,
    OpenCuica = 79,
    HiMidTom = 48,
    LowConga = 64,
    MuteTriangle = 80,
    CrashCymbal1 = 49,
    HighTimbale = 65,
    OpenTriangle = 81,
    HighTom = 50,
    LowTimbale = 66,
}
