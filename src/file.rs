use alloc::fmt;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ops;
use core::str;

#[cfg(not(feature = "std"))]
use micromath::F32Ext;

#[cfg(feature = "std")]
use std::error;

use super::{
    util::*, Channel, HighResTimeCode, MidiMsg, ParseError, ReceiverContext, SystemExclusiveMsg,
    TimeCodeType,
};

// Standard Midi File 1.0 (SMF): RP-001 support

/// Errors that can occur when parsing a [`MidiFile`]
#[derive(Debug, PartialEq)]
pub struct MidiFileParseError {
    pub error: ParseError,
    pub file: MidiFile,
    pub offset: usize,
    pub parsing: String,
    pub remaining_bytes: usize,
    pub next_bytes: Vec<u8>,
}

#[cfg(feature = "std")]
impl error::Error for MidiFileParseError {}

impl fmt::Display for MidiFileParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error parsing MIDI file at position {}: {}",
            &self.offset, &self.error
        )?;
        write!(
            f,
            "\nEncountered this error while parsing: {}",
            &self.parsing
        )?;
        write!(
            f,
            "\nThe incomplete MidiFile that managed to be parsed: {:?}",
            &self.file
        )?;
        write!(
            f,
            "\n\n{} bytes remain in the file. These are the next ones: {:x?}",
            &self.remaining_bytes, &self.next_bytes
        )?;

        Ok(())
    }
}

/// A Standard Midi File (SMF)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MidiFile {
    /// The header chunk: Contains the file format, number of tracks, and division
    pub header: Header,
    /// The track chunks: Contains the actual midi events
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

    fn slice(&self, range: ops::Range<usize>) -> &[u8] {
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
    /// Turn a series of bytes into a `MidiFile`.
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

    /// Turn a `MidiFile` into a series of bytes.
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.header.extend_midi(&mut r);
        for track in &self.tracks {
            track.extend_midi(&mut r);
        }
        r
    }

    /// Add a track to the file. Increments the `num_tracks` field in the header.
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.header.num_tracks += 1;
    }

    /// Remove a track from the file. Decrements the `num_tracks` field in the header.
    pub fn remove_track(&mut self, track_num: usize) {
        self.tracks.remove(track_num);
        self.header.num_tracks -= 1;
    }

    /// Add a midi event to a track in the file, given its absolute beat or frame time. The event delta time is calculated from the previous event in the track and the time division of the file.
    pub fn extend_track(&mut self, track_num: usize, event: MidiMsg, beat_or_frame: f32) {
        match &mut self.tracks[track_num] {
            Track::Midi(events) => {
                let last_beat_or_frame = events.last().map(|e| e.beat_or_frame).unwrap_or(0.0);
                let last_event_tick = self
                    .header
                    .division
                    .beat_or_frame_to_tick(last_beat_or_frame);
                let this_event_tick = self.header.division.beat_or_frame_to_tick(beat_or_frame);
                events.push(TrackEvent {
                    delta_time: this_event_tick - last_event_tick,
                    event,
                    beat_or_frame,
                })
            }

            Track::AlienChunk(_) => panic!("Cannot extend an alien chunk"),
        }
    }
}

/// The header chunk of a Standard Midi File
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Header {
    /// The format of the file
    pub format: SMFFormat,
    /// How many tracks are in the file
    pub num_tracks: u16,
    /// The "division" of the file, which specifies the meaning of the delta times in the file
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

/// The format of a Standard Midi File
#[derive(Debug, Clone, PartialEq)]
pub enum SMFFormat {
    /// A single track file
    SingleTrack,
    /// The file contains multiple tracks, but they are all meant to be played simultaneously
    MultiTrack,
    /// The file contains multiple tracks, but they are independent of each other
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

/// The division of a Standard Midi File, which specifies the meaning of the delta times in the file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Division {
    /// Metrical time. Number of "ticks" per quarter note.
    TicksPerQuarterNote(u16),
    /// Time code based time.
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

impl Division {
    /// Convert a beat or frame to the number of "ticks" in a file with this division.
    pub fn beat_or_frame_to_tick(&self, beat_or_frame: f32) -> u32 {
        match self {
            Division::TicksPerQuarterNote(tpqn) => (beat_or_frame * *tpqn as f32) as u32,
            Division::TimeCode {
                ticks_per_frame, ..
            } => (beat_or_frame * *ticks_per_frame as f32) as u32,
        }
    }

    /// Convert a number of file "ticks" to a beat or frame in a file with this division.
    pub fn ticks_to_beats_or_frames(&self, ticks: u32) -> f32 {
        match self {
            Division::TicksPerQuarterNote(tpqn) => ticks as f32 / *tpqn as f32,
            Division::TimeCode {
                ticks_per_frame, ..
            } => ticks as f32 / *ticks_per_frame as f32,
        }
    }
}

/// A track in a Standard Midi File
#[derive(Debug, Clone, PartialEq)]
pub enum Track {
    /// A standard "MTrk" chunk
    Midi(Vec<TrackEvent>),
    /// Any other chunk data.
    ///
    /// This includes the entire chuck data, include whatever chunk type and length.
    AlienChunk(Vec<u8>),
}

impl Default for Track {
    fn default() -> Self {
        Track::Midi(vec![])
    }
}

impl Track {
    /// Get the number of events in the track, or the length in bytes of an `AlienChunk`.
    pub fn len(&self) -> usize {
        match self {
            Track::Midi(events) => events.len(),
            Track::AlienChunk(data) => data.len(),
        }
    }

    /// Get the [`TrackEvent`] events in the track. Will be empty for an `AlienChunk`.
    pub fn events(&self) -> &[TrackEvent] {
        match self {
            Track::Midi(events) => events,
            Track::AlienChunk(_) => &[],
        }
    }

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
        let reciever_ctx = &mut ReceiverContext::default().parsing_smf();

        let mut i = 0;
        let mut last_beat_or_frame = 0.0;
        while ctx.offset < ctx.track_end {
            ctx.parsing(format!("track {} event {}", track_num, i));
            let (event, event_len) = TrackEvent::from_midi(
                ctx.data(),
                reciever_ctx,
                &ctx.file.header.division,
                last_beat_or_frame,
            )?;
            last_beat_or_frame = event.beat_or_frame;
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

/// An event occurring in a track in a Standard Midi File
#[derive(Debug, Clone, PartialEq)]
pub struct TrackEvent {
    /// The time since the last event. The meaning of this value is determined by the file header's [`Division`].
    pub delta_time: u32,
    /// The actual midi event.
    pub event: MidiMsg,
    /// Given the file's [`Division`], the time in beats or frames at which this event occurs.
    ///
    /// When deserializing, this is derived from the `delta_time` and the previous event's `beat_or_frame`.
    ///
    /// When manually constructing `TrackEvent`s (i.e. when not using the [`MidiFile::extend_track`] convenience function), this field can set to any value, as it is not used when serializing the file.
    pub beat_or_frame: f32,
}

impl TrackEvent {
    fn from_midi(
        v: &[u8],
        ctx: &mut ReceiverContext,
        division: &Division,
        last_beat_or_frame: f32,
    ) -> Result<(Self, usize), ParseError> {
        let (delta_time, time_offset) = read_vlq(v)?;
        let beat_or_frame = last_beat_or_frame + division.ticks_to_beats_or_frames(delta_time);
        match v[time_offset..].first() {
            Some(b) => match b >> 4 {
                0xF => match b & 0b0000_1111 {
                    0x0 => {
                        let (len, len_offset) = read_vlq(&v[time_offset + 1..])?;
                        let p = time_offset + len_offset + 1;
                        ctx.is_smf_sysex = true;
                        let (event, event_len) = SystemExclusiveMsg::from_midi(&v[p..], ctx)?;
                        // event_length does not include the terminating 0xF7 byte, while len is the length of the entire message
                        if event_len != len as usize + 1 {
                            return Err(ParseError::Invalid("Invalid system exclusive message"));
                        }
                        Ok((
                            Self {
                                delta_time,
                                event: MidiMsg::SystemExclusive { msg: event },
                                beat_or_frame,
                            },
                            p + len as usize,
                        ))
                    }
                    0x7 => {
                        let (len, len_offset) = read_vlq(&v[time_offset + 1..])?;
                        let p = time_offset + len_offset + 1;
                        ctx.is_smf_sysex = false;
                        let (event, event_len) = MidiMsg::from_midi_with_context(&v[p..], ctx)?;

                        if event_len != len as usize + 1 {
                            return Err(ParseError::Invalid("Invalid system exclusive message"));
                        }
                        Ok((
                            Self {
                                delta_time,
                                event,
                                beat_or_frame,
                            },
                            p + len as usize,
                        ))
                    }
                    0xF => {
                        let p = time_offset + 1;
                        let (event, event_len) = Meta::from_midi(&v[p..])?;
                        Ok((
                            Self {
                                delta_time,
                                event: MidiMsg::Meta { msg: event },
                                beat_or_frame,
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
                    Ok((
                        Self {
                            delta_time,
                            event,
                            beat_or_frame,
                        },
                        time_offset + event_len,
                    ))
                }
            },
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn extend_midi(&self, v: &mut Vec<u8>) {
        if matches!(
            self.event,
            MidiMsg::SystemRealTime {
                msg: crate::SystemRealTimeMsg::SystemReset,
            }
        ) {
            #[cfg(feature = "std")]
            log::warn!("SMF contains System Reset event, which is not valid. Skipping.");
            return;
        }

        push_vlq(self.delta_time, v);
        // TODO this doesn't handle running-status events
        let event = self.event.to_midi();

        let is_meta = matches!(self.event, MidiMsg::Meta { .. });
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

/// A meta event in a Standard Midi File
#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    /// Must occur at the start of a track, and specifies the sequence number of the track. In a MultiSong file, this is the "pattern" number that identifies the song for cueing purposes.
    SequenceNumber(u16),
    /// Any text, describing anything
    Text(String),
    /// A copyright notice
    Copyright(String),
    /// The name of the track
    TrackName(String),
    /// The name of the instrument used in the track
    InstrumentName(String),
    /// A lyric. See RP-017 for guidance on the use of this meta event.
    Lyric(String),
    /// Normally only used in a SingleTrack file, or the first track of a MultiTrack file. Used to mark significant points in the music.
    Marker(String),
    /// A description of something happening at a point in time
    CuePoint(String),
    /// The MIDI channel that the following track events are intended for. Effective until the next event that specifies a channel.
    ChannelPrefix(Channel),
    /// Marks the end of a track. This event is not optional. It must be the last event in every track.
    EndOfTrack,
    /// The tempo in microseconds per quarter note.
    SetTempo(u32),
    /// If present, the time at which the track is supposed to start. Should be present at the start of the track.
    SmpteOffset(HighResTimeCode),
    /// A time signature.
    TimeSignature(FileTimeSignature),
    /// A key signature.
    KeySignature(KeySignature),
    /// A chunk of data that is specific to the sequencer that created the file.
    SequencerSpecific(Vec<u8>),
    // TODO: RP-32
    // TODO: RP-19
    /// Any other meta event that is not recognized
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
            0x20 => Ok((Self::ChannelPrefix(Channel::from_u8(data[0])), end)),
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
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::Copyright(s) => {
                v.push(0x02);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::TrackName(s) => {
                v.push(0x03);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::InstrumentName(s) => {
                v.push(0x04);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::Lyric(s) => {
                v.push(0x05);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::Marker(s) => {
                v.push(0x06);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::CuePoint(s) => {
                v.push(0x07);
                let bytes = s.as_bytes();
                push_vlq(bytes.len() as u32, v);
                v.extend_from_slice(bytes);
            }
            Meta::ChannelPrefix(n) => {
                v.push(0x20);
                push_vlq(1, v);
                v.push(*n as u8);
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

/// A time signature occurring in a Standard Midi File.
#[derive(Debug, Clone, PartialEq)]
pub struct FileTimeSignature {
    /// The numerator of the time signature, as it would be notated.
    pub numerator: u8,
    /// The denominator of the time signature, as it would be notated.
    ///
    /// This is tranformed from the power-of-two representation used in the file.
    pub denominator: u16,
    /// The number of MIDI clocks per metronome tick.
    pub clocks_per_metronome_tick: u8,
    /// How many 32nd notes are in a MIDI quarter note, which should usually be 8.
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

/// A key signature occurring in a Standard Midi File.
#[derive(Debug, Clone, PartialEq)]
pub struct KeySignature {
    /// Negative for number of flats, positive for number of sharps
    pub key: i8,
    /// 0 for major, 1 for minor
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_time_signature() {
        let midi_data = vec![4, 2, 24, 8];
        let time_sig = FileTimeSignature::from_midi(&midi_data).unwrap();

        assert_eq!(time_sig.numerator, 4);
        assert_eq!(time_sig.denominator, 4);
        assert_eq!(time_sig.clocks_per_metronome_tick, 24);
        assert_eq!(time_sig.thirty_second_notes_per_24_clocks, 8);

        let mut output = Vec::new();
        time_sig.extend_midi(&mut output);
        assert_eq!(output, midi_data);
    }

    #[test]
    fn test_file_time_signature_error() {
        let midi_data = vec![4, 2, 24];
        assert!(matches!(
            FileTimeSignature::from_midi(&midi_data),
            Err(ParseError::UnexpectedEnd)
        ));
    }

    #[test]
    fn test_key_signature() {
        let midi_data = vec![2, 0];
        let key_sig = KeySignature::from_midi(&midi_data).unwrap();

        assert_eq!(key_sig.key, 2);
        assert_eq!(key_sig.scale, 0);

        let mut output = Vec::new();
        key_sig.extend_midi(&mut output);
        assert_eq!(output, midi_data);
    }

    #[test]
    fn test_key_signature_error() {
        let midi_data = vec![2];
        assert!(matches!(
            KeySignature::from_midi(&midi_data),
            Err(ParseError::UnexpectedEnd)
        ));
    }

    #[test]
    fn test_file_serde() {
        use crate::message::MidiMsg;
        use crate::Channel;
        use crate::ChannelVoiceMsg;

        // Create a simple MIDI file
        let mut file = MidiFile::default();
        // Set the division
        file.header.division = Division::TicksPerQuarterNote(480);

        file.add_track(Track::default());

        // Add some events to the track
        file.extend_track(
            0,
            MidiMsg::Meta {
                msg: Meta::TrackName("Test Track".to_string()),
            },
            0.0,
        );

        file.extend_track(
            0,
            MidiMsg::Meta {
                msg: Meta::TimeSignature(FileTimeSignature {
                    numerator: 4,
                    denominator: 4,
                    clocks_per_metronome_tick: 24,
                    thirty_second_notes_per_24_clocks: 8,
                }),
            },
            0.0,
        );

        file.extend_track(
            0,
            MidiMsg::ChannelVoice {
                channel: Channel::Ch1,
                msg: ChannelVoiceMsg::NoteOn {
                    note: 60,
                    velocity: 64,
                },
            },
            0.0,
        );

        file.extend_track(
            0,
            MidiMsg::ChannelVoice {
                channel: Channel::Ch1,
                msg: ChannelVoiceMsg::NoteOff {
                    note: 60,
                    velocity: 64,
                },
            },
            1.0,
        );
        file.extend_track(
            0,
            MidiMsg::Meta {
                msg: Meta::EndOfTrack,
            },
            2.0,
        );

        // Convert the file to bytes
        let bytes = file.to_midi();

        // Assert that we've created a valid MIDI file
        assert!(bytes.starts_with(b"MThd"));
        assert!(bytes[14..].starts_with(b"MTrk"));

        let deserialized_file = MidiFile::from_midi(&bytes).unwrap();
        assert_eq!(deserialized_file.tracks.len(), 1);
        assert_eq!(deserialized_file.tracks[0].events().len(), 5);
        assert_eq!(
            deserialized_file.header.division,
            Division::TicksPerQuarterNote(480)
        );
        assert_eq!(deserialized_file, file);
    }

    #[test]
    fn test_file_system_reset() {
        let mut file = MidiFile::default();
        file.add_track(Track::default());
        file.extend_track(
            0,
            MidiMsg::SystemRealTime {
                msg: crate::SystemRealTimeMsg::SystemReset,
            },
            0.0,
        );
        let bytes = file.to_midi();

        let deserialized_file = MidiFile::from_midi(&bytes).unwrap();
        assert_eq!(deserialized_file.tracks.len(), 1);
        // The system reset message should not be included in the track, since it is not a valid MIDI file message
        assert_eq!(deserialized_file.tracks[0].events().len(), 0);
    }
}
