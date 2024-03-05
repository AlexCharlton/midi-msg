use std::str;

use super::{
    util::*, HighResTimeCode, MidiMsg, ParseError, ReceiverContext, SystemExclusiveMsg,
    TimeCodeType,
};

/// Standard Midi File 1.0 (SMF): RP-001 support

#[derive(Debug, PartialEq)]
pub struct MidiFileParseError {
    pub error: ParseError,
    pub file: MidiFile,
    pub offset: usize,
    pub parsing: String,
    pub remaining_bytes: usize,
    pub next_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiFile {
    pub header: Header,
    pub tracks: Vec<Track>,
}

#[derive(Debug)]
struct ParseCtx<'a, 'b> {
    input: &'a [u8],
    offset: usize,
    parsing: String,
    file: &'b mut MidiFile,
    track_end: usize,
}

impl<'a, 'b> ParseCtx<'a, 'b> {
    fn new(input: &'a [u8], file: &'b mut MidiFile) -> Self {
        Self {
            input,
            offset: 0,
            parsing: "header".into(),
            file,
            track_end: 0,
        }
    }

    fn advance(&mut self, offset: usize) {
        self.offset += offset;
    }

    fn data(&self) -> &[u8] {
        &self.input[self.offset..]
    }

    fn slice(&self, range: std::ops::Range<usize>) -> &[u8] {
        &self.input[range.start + self.offset..range.end + self.offset]
    }

    fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    fn parsing<S: Into<String>>(&mut self, s: S) {
        self.parsing = s.into();
    }

    fn add_track(&mut self, track: Track) {
        self.file.tracks.push(track);
    }

    fn track_length(&mut self, len: usize) {
        self.track_end = self.offset + len;
    }

    fn extend_track(&mut self, event: TrackEvent) {
        self.file.tracks.last_mut().unwrap().extend(event);
    }
}

impl MidiFile {
    pub fn from_midi(v: &[u8]) -> Result<Self, MidiFileParseError> {
        let mut file = MidiFile {
            header: Header::default(),
            tracks: vec![],
        };
        let mut ctx = ParseCtx::new(v, &mut file);
        match Header::parse_midi_file(&mut ctx) {
            Ok(_) => (),
            Err(error) => {
                let offset = ctx.offset;
                let parsing = ctx.parsing.clone();
                let remaining_bytes = ctx.remaining();
                let next_bytes = ctx.slice(0..(20.min(ctx.remaining()))).to_vec();
                return Err(MidiFileParseError {
                    error,
                    file,
                    offset,
                    parsing,
                    remaining_bytes,
                    next_bytes,
                });
            }
        }

        for i in 0..ctx.file.header.num_tracks {
            match Track::parse_midi_file(&mut ctx, i) {
                Ok(_) => (),
                Err(error) => {
                    let offset = ctx.offset;
                    let parsing = ctx.parsing.clone();
                    let remaining_bytes = ctx.remaining();
                    let next_bytes = ctx.slice(0..(20.min(ctx.remaining()))).to_vec();
                    return Err(MidiFileParseError {
                        error,
                        file,
                        offset,
                        parsing,
                        remaining_bytes,
                        next_bytes,
                    });
                }
            }
        }
        Ok(file)
    }

    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        self.header.extend_midi(v);
        for track in &self.tracks {
            track.extend_midi(v);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Header {
    pub format: SMFFormat,
    pub num_tracks: u16,
    pub division: Division,
}

impl Header {
    // We pass the file to the from_midi function, so that we can have a running context of what's been parsed so far.
    fn parse_midi_file(ctx: &mut ParseCtx) -> Result<(), ParseError> {
        if ctx.remaining() < 14 {
            return Err(ParseError::UnexpectedEnd);
        }
        if str::from_utf8(ctx.slice(0..4)) != Ok("MThd") {
            return Err(ParseError::Invalid("Invalid header"));
        }
        ctx.advance(4);
        if u32_from_midi(ctx.slice(0..4)) != Ok(6) {
            return Err(ParseError::Invalid("Invalid header length"));
        }
        ctx.advance(4);
        let v = ctx.data();

        let (format, _) = SMFFormat::from_midi(v)?;
        let num_tracks = u16::from_be_bytes([v[2], v[3]]);
        let division = if v[4] & 0b1000_0000 == 0 {
            Division::TicksPerQuarterNote(u16::from_be_bytes([v[4], v[5]]))
        } else {
            Division::TimeCode {
                frames_per_second: match v[4] & 0b0111_1111 {
                    0 => TimeCodeType::FPS24,
                    1 => TimeCodeType::FPS25,
                    2 => TimeCodeType::DF30,
                    3 => TimeCodeType::NDF30,
                    _ => return Err(ParseError::Invalid("Invalid time code type")),
                },
                ticks_per_frame: v[5],
            }
        };
        ctx.advance(6);
        ctx.file.header = Self {
            format,
            num_tracks,
            division,
        };
        Ok(())
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        v.extend_from_slice(b"MThd");
        push_u32(6, v); // Length of header, always 6 bytes

        self.format.extend_midi(v);
        push_u16(self.num_tracks, v);
        match self.division {
            Division::TicksPerQuarterNote(tpqn) => {
                push_u16(tpqn, v);
            }
            Division::TimeCode {
                frames_per_second,
                ticks_per_frame,
            } => {
                v.push(0b1000_0000 | frames_per_second as u8);
                v.push(ticks_per_frame);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SMFFormat {
    SingleTrack,
    MultiTrack,
    MultiSong,
}

impl Default for SMFFormat {
    fn default() -> Self {
        SMFFormat::MultiTrack
    }
}

impl SMFFormat {
    fn from_midi(v: &[u8]) -> Result<(Self, usize), ParseError> {
        if v.len() < 2 {
            return Err(ParseError::UnexpectedEnd);
        }
        Ok((
            // Big endian 16 bit value: Only the LSB is used
            match v[1] {
                0 => SMFFormat::SingleTrack,
                1 => SMFFormat::MultiTrack,
                2 => SMFFormat::MultiSong,
                _ => return Err(ParseError::Invalid("Invalid SMF format")),
            },
            2,
        ))
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            SMFFormat::SingleTrack => v.extend_from_slice(&[0, 0]),
            SMFFormat::MultiTrack => v.extend_from_slice(&[0, 1]),
            SMFFormat::MultiSong => v.extend_from_slice(&[0, 2]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Division {
    // Metrical time. Number of "ticks" per quarter note.
    TicksPerQuarterNote(u16),
    TimeCode {
        frames_per_second: TimeCodeType,
        ticks_per_frame: u8,
    },
}

impl Default for Division {
    fn default() -> Self {
        Division::TicksPerQuarterNote(96)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    // Standard MTrk chunk
    Midi(Vec<TrackEvent>),
    // Any other chunk data
    // This includes the entire chuck data, include whatever chunk type and length
    AlienChunk(Vec<u8>),
}

impl Track {
    fn extend(&mut self, event: TrackEvent) {
        match self {
            Track::Midi(events) => events.push(event),
            Track::AlienChunk(_) => panic!("Cannot extend an alien chunk"),
        }
    }

    fn parse_midi_file(ctx: &mut ParseCtx, track_num: u16) -> Result<(), ParseError> {
        if ctx.remaining() < 8 {
            return Err(ParseError::UnexpectedEnd);
        }
        ctx.parsing(format!("track {}", track_num));
        let len = u32_from_midi(ctx.slice(4..8))? as usize;
        if ctx.remaining() < len + 8 {
            return Err(ParseError::UnexpectedEnd);
        }
        if str::from_utf8(ctx.slice(0..4)) != Ok("MTrk") {
            ctx.add_track(Self::AlienChunk(ctx.slice(0..len + 8).to_vec()));
            ctx.advance(len + 8);
            return Ok(());
        }
        ctx.add_track(Self::Midi(vec![]));
        ctx.advance(8);
        ctx.track_length(len);
        let reciever_ctx = &mut ReceiverContext::default();

        let mut i = 0;
        while ctx.offset < ctx.track_end {
            ctx.parsing(format!("track {} event {}", track_num, i));
            let (event, event_len) = TrackEvent::from_midi(ctx.data(), reciever_ctx)?;
            ctx.extend_track(event);
            ctx.advance(event_len);
            i += 1;
        }
        if ctx.offset > ctx.track_end {
            return Err(ParseError::Invalid(
                "Track length exceeded the provided length",
            ));
        }
        Ok(())
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Track::Midi(events) => {
                v.extend_from_slice(b"MTrk");
                let s = v.len();
                push_u32(0, v); // We will fill this in after we know the length
                for event in events {
                    event.extend_midi(v);
                }
                let e = v.len();
                // Fill in the length
                v[s..s + 4].copy_from_slice(&(e as u32 - s as u32 - 4).to_be_bytes());
            }
            Track::AlienChunk(data) => {
                v.extend_from_slice(&data);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackEvent {
    pub delta_time: u32,
    pub event: MidiMsg,
}

impl TrackEvent {
    fn from_midi(v: &[u8], ctx: &mut ReceiverContext) -> Result<(Self, usize), ParseError> {
        let (delta_time, time_offset) = read_vlq(v)?;
        match v[time_offset..].first() {
            Some(b) => match b >> 4 {
                0xF => match b & 0b0000_1111 {
                    0x0 => {
                        let (len, len_offset) = read_vlq(&v[time_offset + 1..])?;
                        let p = time_offset + len_offset + 1;
                        ctx.is_smf_sysex = true;
                        let (event, event_len) = SystemExclusiveMsg::from_midi(&v[p..], ctx)?;
                        if event_len != len as usize {
                            return Err(ParseError::Invalid("Invalid system exclusive message"));
                        }
                        Ok((
                            Self {
                                delta_time,
                                event: MidiMsg::SystemExclusive { msg: event },
                            },
                            p + event_len,
                        ))
                    }
                    0x7 => {
                        let (len, len_offset) = read_vlq(&v[time_offset + 1..])?;
                        let p = time_offset + len_offset + 1;
                        ctx.is_smf_sysex = false;
                        let (event, event_len) = MidiMsg::from_midi_with_context(&v[p..], ctx)?;

                        if event_len != len as usize {
                            return Err(ParseError::Invalid("Invalid system exclusive message"));
                        }
                        Ok((Self { delta_time, event }, p + event_len))
                    }
                    0xF => {
                        let p = time_offset + 1;
                        let (event, event_len) = Meta::from_midi(&v[p..])?;
                        Ok((
                            Self {
                                delta_time,
                                event: MidiMsg::Meta { msg: event },
                            },
                            p + event_len,
                        ))
                    }
                    _ => Err(ParseError::Invalid("Invalid track event")),
                },
                _ => {
                    ctx.is_smf_sysex = false;
                    let (event, event_len) =
                        MidiMsg::from_midi_with_context(&v[time_offset..], ctx)?;
                    Ok((Self { delta_time, event }, time_offset + event_len))
                }
            },
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        push_vlq(self.delta_time, v);
        let event = self.event.to_midi();

        let is_meta = match self.event {
            MidiMsg::Meta { .. } => true,
            _ => false,
        };
        // Any kind of system event
        let is_system = match self.event {
            MidiMsg::SystemExclusive { .. }
            | MidiMsg::SystemCommon { .. }
            | MidiMsg::SystemRealTime { .. } => true,
            _ => false,
        };
        if is_meta {
            v.push(0xFF);
        } else if is_system {
            // We always use the 0xF7 format for system events, since it can be used for all system events, not just system exclusive
            v.push(0xF7);
            push_vlq(event.len() as u32, v);
        }
        v.extend_from_slice(&event);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    SequenceNumber(u16),
    Text(String),
    Copyright(String),
    TrackName(String),
    InstrumentName(String),
    Lyric(String),
    Marker(String),
    CuePoint(String),
    ChannelPrefix(u8),
    EndOfTrack,
    // Microseconds per quarter note
    SetTempo(u32),
    SmpteOffset(HighResTimeCode),
    TimeSignature(FileTimeSignature),
    KeySignature(KeySignature),
    SequencerSpecific(Vec<u8>),
    Unknown { meta_type: u8, data: Vec<u8> },
}

impl Meta {
    // We do not extend with 0xFF, as this is done in TrackEvent::extend_midi
    pub(crate) fn from_midi(v: &[u8]) -> Result<(Self, usize), ParseError> {
        if v.len() < 2 {
            return Err(ParseError::UnexpectedEnd);
        }
        let meta_type = v[0];
        let (len, len_offset) = read_vlq(&v[1..])?;
        if v.len() < len as usize + len_offset + 1 {
            return Err(ParseError::UnexpectedEnd);
        }
        let end = len as usize + len_offset + 1;
        let data = &v[len_offset + 1..end];
        match meta_type {
            0x00 => Ok((
                Self::SequenceNumber(u16::from_be_bytes([data[0], data[1]])),
                end,
            )),
            0x01 => Ok((Self::Text(String::from_utf8_lossy(data).to_string()), end)),
            0x02 => Ok((
                Self::Copyright(String::from_utf8_lossy(data).to_string()),
                end,
            )),
            0x03 => Ok((
                Self::TrackName(String::from_utf8_lossy(data).to_string()),
                end,
            )),
            0x04 => Ok((
                Self::InstrumentName(String::from_utf8_lossy(data).to_string()),
                end,
            )),
            0x05 => Ok((Self::Lyric(String::from_utf8_lossy(data).to_string()), end)),
            0x06 => Ok((Self::Marker(String::from_utf8_lossy(data).to_string()), end)),
            0x07 => Ok((
                Self::CuePoint(String::from_utf8_lossy(data).to_string()),
                end,
            )),
            0x20 => Ok((Self::ChannelPrefix(data[0]), end)),
            0x2F => Ok((Self::EndOfTrack, end)),
            0x51 => Ok((
                Self::SetTempo(u32::from_be_bytes([0, data[0], data[1], data[2]])),
                end,
            )),
            0x54 => {
                let (time, _) = HighResTimeCode::from_midi(data)?;
                Ok((Self::SmpteOffset(time), end))
            }
            0x58 => Ok((
                Self::TimeSignature(FileTimeSignature::from_midi(data)?),
                end,
            )),
            0x59 => Ok((Self::KeySignature(KeySignature::from_midi(data)?), end)),
            0x7F => Ok((Self::SequencerSpecific(data.to_vec()), end)),
            _ => Ok((
                Self::Unknown {
                    meta_type,
                    data: data.to_vec(),
                },
                end,
            )),
        }
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Meta::SequenceNumber(n) => {
                v.push(0x00);
                push_vlq(2, v);
                v.extend_from_slice(&n.to_be_bytes());
            }
            Meta::Text(s) => {
                v.push(0x01);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::Copyright(s) => {
                v.push(0x02);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::TrackName(s) => {
                v.push(0x03);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::InstrumentName(s) => {
                v.push(0x04);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::Lyric(s) => {
                v.push(0x05);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::Marker(s) => {
                v.push(0x06);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::CuePoint(s) => {
                v.push(0x07);
                push_vlq(s.len() as u32, v);
                v.extend_from_slice(s.as_bytes());
            }
            Meta::ChannelPrefix(n) => {
                v.push(0x20);
                push_vlq(1, v);
                v.push(*n);
            }
            Meta::EndOfTrack => {
                v.push(0x2F);
                push_vlq(0, v);
            }
            Meta::SetTempo(n) => {
                v.push(0x51);
                push_vlq(3, v);
                v.extend_from_slice(&n.to_be_bytes()[1..]);
            }
            Meta::SmpteOffset(t) => {
                v.push(0x54);
                push_vlq(5, v);
                t.extend_midi(v);
            }
            Meta::TimeSignature(s) => {
                v.push(0x58);
                push_vlq(4, v);
                s.extend_midi(v);
            }
            Meta::KeySignature(k) => {
                v.push(0x59);
                push_vlq(2, v);
                k.extend_midi(v);
            }
            Meta::SequencerSpecific(d) => {
                v.push(0x7F);
                push_vlq(d.len() as u32, v);
                v.extend_from_slice(d);
            }
            Meta::Unknown { meta_type, data } => {
                v.push(*meta_type);
                push_vlq(data.len() as u32, v);
                v.extend_from_slice(data);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileTimeSignature {
    pub numerator: u8,
    pub denominator: u16,
    pub clocks_per_metronome_tick: u8,
    pub thirty_second_notes_per_24_clocks: u8,
}

impl FileTimeSignature {
    pub(crate) fn from_midi(m: &[u8]) -> Result<Self, ParseError> {
        if m.len() < 4 {
            return Err(ParseError::UnexpectedEnd);
        }
        Ok(Self {
            numerator: m[0],
            denominator: u16::pow(2, m[1] as u32),
            clocks_per_metronome_tick: m[2],
            thirty_second_notes_per_24_clocks: m[3],
        })
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.numerator);
        v.push((self.denominator as f32).log2() as u8);
        v.push(self.clocks_per_metronome_tick);
        v.push(self.thirty_second_notes_per_24_clocks);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeySignature {
    // Negative for number of flats, positive for number of sharps
    pub key: i8,
    // 0 for major, 1 for minor
    pub scale: u8,
}

impl KeySignature {
    pub(crate) fn from_midi(m: &[u8]) -> Result<Self, ParseError> {
        if m.len() < 2 {
            return Err(ParseError::UnexpectedEnd);
        }
        Ok(Self {
            key: m[0] as i8,
            scale: m[1],
        })
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.key as u8);
        v.push(self.scale);
    }
}
