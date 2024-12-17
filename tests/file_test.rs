use midi_msg::*;

#[test]
#[cfg(feature = "file")]
fn test_smf_file() {
    let test1 = include_bytes!("./test1.mid");
    let expected = MidiFile {
        header: Header {
            format: SMFFormat::MultiTrack,
            num_tracks: 2,
            division: Division::TicksPerQuarterNote(192),
        },
        tracks: vec![
            Track::Midi(vec![
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::TimeSignature(FileTimeSignature {
                            numerator: 4,
                            denominator: 4,
                            clocks_per_metronome_tick: 24,
                            thirty_second_notes_per_24_clocks: 8,
                        }),
                    },
                    beat_or_frame: 0.0,
                },
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::SetTempo(500000),
                    },
                    beat_or_frame: 0.0,
                },
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::TrackName("Tempo Track".to_string()),
                    },
                    beat_or_frame: 0.0,
                },
                TrackEvent {
                    delta_time: 7680,
                    event: MidiMsg::Meta {
                        msg: Meta::EndOfTrack,
                    },
                    beat_or_frame: 40.0,
                },
            ]),
            Track::Midi(vec![
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::TrackName("New Instrument".to_string()),
                    },
                    beat_or_frame: 0.0,
                },
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: ChannelVoiceMsg::NoteOn {
                            note: 60,
                            velocity: 100,
                        },
                    },
                    beat_or_frame: 0.0,
                },
                TrackEvent {
                    delta_time: 768,
                    event: MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: ChannelVoiceMsg::NoteOff {
                            note: 60,
                            velocity: 0,
                        },
                    },
                    beat_or_frame: 4.0,
                },
                TrackEvent {
                    delta_time: 6912,
                    event: MidiMsg::Meta {
                        msg: Meta::EndOfTrack,
                    },
                    beat_or_frame: 40.0,
                },
            ]),
        ],
    };
    let deserialize_result = MidiFile::from_midi(test1);
    assert!(deserialize_result.is_ok());
    assert_eq!(deserialize_result.unwrap(), expected);

    let serialized = expected.to_midi();
    assert_eq!(&serialized, test1);
}

#[test]
#[cfg(feature = "file")]
fn test_score_file() {
    // File generated from MuseScore 4. The file is a simple score with a single track, but MuseScore adds a number of meta events and control changes.
    let test_score1 = include_bytes!("./test_score1.mid");
    let expected = MidiFile {
        header: Header {
            format: SMFFormat::MultiTrack,
            num_tracks: 1,
            division: Division::TicksPerQuarterNote(480),
        },
        tracks: vec![Track::Midi(vec![
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::Meta {
                    msg: Meta::TrackName("Piccolo".to_string()),
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::Meta {
                    msg: Meta::TimeSignature(FileTimeSignature {
                        numerator: 4,
                        denominator: 4,
                        clocks_per_metronome_tick: 24,
                        thirty_second_notes_per_24_clocks: 8,
                    }),
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::Meta {
                    msg: Meta::KeySignature(KeySignature { key: 0, scale: 0 }),
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::Meta {
                    msg: Meta::SetTempo(500000),
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelMode {
                    channel: Channel::Ch1,
                    msg: ChannelModeMsg::ResetAllControllers,
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 100,
                            value: 0,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 101,
                            value: 0,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 6,
                            value: 12,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 100,
                            value: 127,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 101,
                            value: 127,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ProgramChange { program: 72 },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 7,
                            value: 100,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 10,
                            value: 64,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 91,
                            value: 0,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 93,
                            value: 0,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::Meta {
                    msg: Meta::Unknown {
                        meta_type: 33,
                        data: vec![0],
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::NoteOn {
                        note: 84,
                        velocity: 80,
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 0,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::ControlChange {
                        control: ControlChange::CC {
                            control: 2,
                            value: 80,
                        },
                    },
                },
                beat_or_frame: 0.0,
            },
            TrackEvent {
                delta_time: 479,
                event: MidiMsg::ChannelVoice {
                    channel: Channel::Ch1,
                    msg: ChannelVoiceMsg::NoteOn {
                        note: 84,
                        velocity: 0,
                    },
                },
                beat_or_frame: 0.99791664,
            },
            TrackEvent {
                delta_time: 1,
                event: MidiMsg::Meta {
                    msg: Meta::EndOfTrack,
                },
                beat_or_frame: 1.0,
            },
        ])],
    };
    let deserialize_result = MidiFile::from_midi(test_score1);
    assert!(deserialize_result.is_ok());
    assert_eq!(deserialize_result.unwrap(), expected);
}

#[test]
#[cfg(feature = "file")]
fn test_smf_file_with_sysex() {
    let test_file = include_bytes!("./breaking-the-law.mid");

    let deserialize_result = MidiFile::from_midi(test_file);
    assert!(deserialize_result.is_ok());

    // Re-serializing does not produce invalid messages
    let file = deserialize_result.unwrap();
    let serialized = file.to_midi();
    let deserialize_result = MidiFile::from_midi(&serialized);
    assert!(deserialize_result.is_ok());
    assert!(!file_contains_invalid_message(deserialize_result.unwrap()));
}

#[test]
#[cfg(feature = "file")]
fn test_smf_files_with_invalid_sysex() {
    // Byte overflow
    let test_file = include_bytes!("./echoes.mid");
    let deserialize_result = MidiFile::from_midi(test_file);
    assert!(deserialize_result.is_ok());
    assert!(file_contains_invalid_message(deserialize_result.unwrap()));

    // Empty sysex
    let test_file = include_bytes!("./shine-on.mid");
    let deserialize_result = MidiFile::from_midi(test_file);
    assert!(deserialize_result.is_ok());
    assert!(file_contains_invalid_message(deserialize_result.unwrap()));

    // Byte overflow
    let test_file = include_bytes!("./1442jsop26.mid");
    let deserialize_result = MidiFile::from_midi(test_file);
    assert!(deserialize_result.is_ok());
    assert!(file_contains_invalid_message(deserialize_result.unwrap()));

    // Empty sysex
    let test_file = include_bytes!("./the-snow-goose.mid");
    let deserialize_result = MidiFile::from_midi(test_file);
    assert!(deserialize_result.is_ok());
    assert!(file_contains_invalid_message(deserialize_result.unwrap()));

    // Serializing does not produce invalid messages
    let file = MidiFile::from_midi(test_file).unwrap();
    let serialized = file.to_midi();
    let deserialize_result = MidiFile::from_midi(&serialized);
    assert!(deserialize_result.is_ok());
    assert!(!file_contains_invalid_message(deserialize_result.unwrap()));
}

fn file_contains_invalid_message(file: MidiFile) -> bool {
    file.tracks
        .iter()
        .any(|track| track.events().iter().any(|event| event.event.is_invalid()))
}
