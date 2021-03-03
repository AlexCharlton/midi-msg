use crate::util::*;
use ascii::{AsciiChar, AsciiString};

/// The set of messages used for accessing files on a shared file system or network
/// so they can be used to play sounds without transferring the file contents.
/// Used by [`UniversalNonRealTimeMsg::FileReference`](crate::UniversalNonRealTimeMsg::FileReference).
///
/// As defined in CA-018.
#[derive(Debug, Clone, PartialEq)]
pub enum FileReferenceMsg {
    /// Describe where a file is located for opening, but must be followed by a `SelectContents`
    /// message if any sounds are to play.
    Open {
        /// A number 0-16383 used to distinguish between multiple file operations on the same device
        ctx: u16,
        file_type: FileReferenceType,
        /// Max 260 character url.
        url: AsciiString,
    },
    /// Given the pointer to a file, prepare it so its sounds can be loaded.
    SelectContents {
        /// A number 0-16383 used to distinguish between multiple file operations on the same device
        ctx: u16,
        /// How to map the file's sounds onto MIDI banks/programs.
        map: SelectMap,
    },
    /// The equivalent of an `Open` and `SelectContents` messages in succession.
    OpenSelectContents {
        /// A number 0-16383 used to distinguish between multiple file operations on the same device
        ctx: u16,
        file_type: FileReferenceType,
        /// Max 260 character url.
        url: AsciiString,
        /// How to map the file's sounds onto MIDI banks/programs.
        map: SelectMap,
    },
    /// Close the file and deallocate the data related to it, such that its sounds should
    /// no longer play.
    Close {
        /// A number 0-16383 used to distinguish between multiple file operations on the same device
        ctx: u16,
    },
}

impl FileReferenceMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Open {
                ctx,
                file_type,
                url,
            } => {
                push_u14(*ctx, v);
                let len = 4 + url.len().min(260) + 1;
                push_u14(len as u16, v);
                file_type.extend_midi(v);
                v.extend_from_slice(&url.as_bytes()[0..url.len().min(260)]);
                v.push(0); // Null terminate URL
            }
            Self::SelectContents { ctx, map } => {
                push_u14(*ctx, v);
                push_u14(map.len() as u16, v);
                map.extend_midi(v);
            }
            Self::OpenSelectContents {
                ctx,
                file_type,
                url,
                map,
            } => {
                push_u14(*ctx, v);
                let len = 4 + url.len().min(260) + 1 + map.len();
                push_u14(len as u16, v);
                file_type.extend_midi(v);
                v.extend_from_slice(&url.as_bytes()[0..url.len().min(260)]);
                v.push(0); // Null terminate URL
                map.extend_midi(v);
            }
            Self::Close { ctx } => {
                push_u14(*ctx, v);
                v.push(0); // Len is zero
                v.push(0); // And here's another byte for some reason ¯\_(ツ)_/¯
            }
        }
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

/// The file type of a given file, as used by [`FileReferenceMsg`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileReferenceType {
    DLS,
    SF2,
    WAV,
}

impl FileReferenceType {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::DLS => {
                v.push(AsciiChar::D.as_byte());
                v.push(AsciiChar::L.as_byte());
                v.push(AsciiChar::S.as_byte());
                v.push(AsciiChar::Space.as_byte());
            }
            Self::SF2 => {
                v.push(AsciiChar::S.as_byte());
                v.push(AsciiChar::F.as_byte());
                v.push(AsciiChar::_2.as_byte());
                v.push(AsciiChar::Space.as_byte());
            }
            Self::WAV => {
                v.push(AsciiChar::W.as_byte());
                v.push(AsciiChar::A.as_byte());
                v.push(AsciiChar::V.as_byte());
                v.push(AsciiChar::Space.as_byte());
            }
        }
    }
}

/// How to map a `DLS` or `SF2` file for MIDI reference. Used by [`SelectMap`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SoundFileMap {
    /// MIDI bank number required to select sound for playing. 0-16383
    pub dst_bank: u16,
    /// MIDI program number required to select sound for playing. 0-127
    pub dst_prog: u8,
    /// MIDI bank number referenced in file's instrument header. 0-16383
    pub src_bank: u16,
    /// MIDI program number referenced in file's instrument header. 0-127
    pub src_prog: u8,
    /// The selected instrument is a drum instrument
    pub src_drum: bool,
    /// The selected instrument should be loaded as a drum instrument
    pub dst_drum: bool,
    /// Initial volume 0-127
    pub volume: u8,
}

impl Default for SoundFileMap {
    fn default() -> Self {
        Self {
            dst_bank: 0,
            dst_prog: 0,
            src_bank: 0,
            src_prog: 0,
            src_drum: false,
            dst_drum: false,
            volume: 0x7F,
        }
    }
}

impl SoundFileMap {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u14(self.dst_bank, v);
        push_u7(self.dst_prog, v);
        push_u14(self.src_bank, v);
        push_u7(self.src_prog, v);
        let mut flags: u8 = 0;
        if self.src_drum {
            flags += 1 << 0;
        }
        if self.dst_drum {
            flags += 1 << 1;
        }
        v.push(flags);
        push_u7(self.volume, v);
    }
}

/// How to map a `WAV` file for MIDI reference. Used by [`SelectMap`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WAVMap {
    /// MIDI bank number required to select sound for playing. 0-16383
    pub dst_bank: u16,
    /// MIDI program number required to select sound for playing. 0-127
    pub dst_prog: u8,
    /// MIDI note where sound plays at original pitch
    pub base: u8,
    /// Lowest MIDI note that plays
    pub lokey: u8,
    /// Highest MIDI note that plays
    pub hikey: u8,
    /// Fine tuning offset -8192-8191, representing the fractional cents to shift
    /// in 1/8192ths of a cent
    pub fine: i16,
    /// Initial volume 0-127
    pub volume: u8,
}

impl WAVMap {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        push_u14(self.dst_bank, v);
        push_u7(self.dst_prog, v);
        push_u7(self.base, v);
        push_u7(self.lokey, v);
        push_u7(self.hikey, v);
        let [msb, lsb] = i_to_u14(self.fine);
        v.push(lsb);
        v.push(msb);
        push_u7(self.volume, v);
    }
}

impl Default for WAVMap {
    fn default() -> Self {
        Self {
            dst_bank: 0,
            dst_prog: 0,
            base: 60,
            lokey: 0,
            hikey: 0x7F,
            fine: 0,
            volume: 0x7F,
        }
    }
}

/// How to map a file for MIDI reference. Used by [`FileReferenceMsg::SelectContents`].
#[derive(Debug, Clone, PartialEq)]
pub enum SelectMap {
    /// Used for DLS or SF2 files. No more than 127 `SoundFileMap`s.
    ///
    /// 0 `SoundFileMap`s indicates "use the map provided in the file".
    SoundFile(Vec<SoundFileMap>),
    /// Used for WAV files.
    WAV(WAVMap),
    /// Used for DLS or SF2 files. Use the mapping provided by the file,
    /// but offset the given MIDI bank by `bank_offset`.
    ///
    /// Defined in CA-028
    SoundFileBankOffset {
        bank_offset: u16,
        /// The selected instrument is a drum instrument
        src_drum: bool,
    },
    /// Used for WAV files. Offset the dest MIDI bank by `bank_offset`.
    ///
    /// Defined in CA-028.
    WAVBankOffset {
        map: WAVMap,
        bank_offset: u16,
        /// The selected instrument is a drum instrument
        src_drum: bool,
    },
}

impl SelectMap {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::WAV(m) => m.extend_midi(v),
            Self::WAVBankOffset {
                map,
                bank_offset,
                src_drum,
            } => {
                map.extend_midi(v);
                v.push(0); // count
                v.push(0); // Extension ID 1
                v.push(1); // Extension ID 2
                v.push(3); // len
                push_u14(*bank_offset, v);
                let mut flags: u8 = 0;
                if *src_drum {
                    flags += 1 << 0;
                }
                push_u7(flags, v);
            }
            Self::SoundFileBankOffset {
                bank_offset,
                src_drum,
            } => {
                v.push(0); // count
                v.push(0); // Extension ID 1
                v.push(1); // Extension ID 2
                v.push(3); // len
                push_u14(*bank_offset, v);
                let mut flags: u8 = 0;
                if *src_drum {
                    flags += 1 << 0;
                }
                push_u7(flags, v);
            }
            Self::SoundFile(maps) => {
                let count = maps.len().min(127);
                push_u7(count as u8, v);
                for m in maps[0..count].iter() {
                    m.extend_midi(v);
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::WAV(_) => 9,
            Self::WAVBankOffset { .. } => 9 + 6,
            Self::SoundFileBankOffset { .. } => 7,
            Self::SoundFile(maps) => {
                let count = maps.len().min(127);
                1 + count * 8
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn serialize_sample_dump_msg() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalNonRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalNonRealTimeMsg::FileReference(
                        FileReferenceMsg::OpenSelectContents {
                            ctx: 44,
                            file_type: FileReferenceType::DLS,
                            url: AsciiString::from_ascii("file://foo.dls").unwrap(),
                            map: SelectMap::SoundFile(vec![SoundFileMap {
                                dst_bank: 1 << 10,
                                src_prog: 1,
                                ..Default::default()
                            }]),
                        }
                    ),
                },
            }
            .to_midi(),
            vec![
                0xF0,
                0x7E,
                0x7F, // All call
                0x0B,
                0x03, // ExtendedSampleDump header
                44,
                00, // ctx
                28,
                0, // len,
                AsciiChar::D.as_byte(),
                AsciiChar::L.as_byte(),
                AsciiChar::S.as_byte(),
                AsciiChar::Space.as_byte(),
                AsciiChar::f.as_byte(),
                AsciiChar::i.as_byte(),
                AsciiChar::l.as_byte(),
                AsciiChar::e.as_byte(),
                // Start URL
                AsciiChar::Colon.as_byte(),
                AsciiChar::Slash.as_byte(),
                AsciiChar::Slash.as_byte(),
                AsciiChar::f.as_byte(),
                AsciiChar::o.as_byte(),
                AsciiChar::o.as_byte(),
                AsciiChar::Dot.as_byte(),
                AsciiChar::d.as_byte(),
                AsciiChar::l.as_byte(),
                AsciiChar::s.as_byte(),
                0, // End of url
                1, // count
                0,
                8, // dst_bank
                0, //dst_prog
                0,
                0,    // src_bank
                1,    // src_prog
                0,    // flags
                0x7f, // vol
                0xF7
            ]
        );
    }
}
