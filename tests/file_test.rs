use midi_msg::*;

#[test]
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
                },
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::SetTempo(500000),
                    },
                },
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::TrackName("Tempo Track".to_string()),
                    },
                },
                TrackEvent {
                    delta_time: 7680,
                    event: MidiMsg::Meta {
                        msg: Meta::EndOfTrack,
                    },
                },
            ]),
            Track::Midi(vec![
                TrackEvent {
                    delta_time: 0,
                    event: MidiMsg::Meta {
                        msg: Meta::TrackName("New Instrument".to_string()),
                    },
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
                },
                TrackEvent {
                    delta_time: 6912,
                    event: MidiMsg::Meta {
                        msg: Meta::EndOfTrack,
                    },
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
