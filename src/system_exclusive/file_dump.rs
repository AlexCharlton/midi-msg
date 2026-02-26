use super::DeviceID;
use crate::parse_error::*;
use crate::util::*;
use crate::Write;
use alloc::vec::Vec;
use bstr::BString;

/// Used to transmit general file data.
/// Used by [`UniversalNonRealTimeMsg`](crate::UniversalNonRealTimeMsg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDumpMsg {
    /// Request that the file with `name` be sent.
    Request {
        requester_device: DeviceID,
        file_type: FileType,
        name: BString,
    },
    /// The header of the file about to be sent.
    Header {
        sender_device: DeviceID,
        file_type: FileType,
        /// Actual (un-encoded) file length, 28 bits (0-2684354561)
        length: u32,
        name: BString,
    },
    /// A packet of the file being sent.
    ///
    /// Use `FileDumpMsg::packet` to construct
    Packet {
        /// Running packet count, 0-127. Wraps back to 0
        running_count: u8,
        /// At most 112 bytes (full 8 bits may be used)
        data: Vec<u8>,
    },
}

#[cfg(feature = "defmt")]
impl defmt::Format for FileDumpMsg {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Self::Header {
                sender_device,
                file_type,
                length,
                name,
            } => {
                defmt::write!(
                    fmt,
                    "FileDumpMsg::Header {{ sender_device: {:?}, file_type: {:?}, length: {}, name: {:?} }}",
                    sender_device, file_type, length, name.as_slice()
                )
            }
            Self::Packet {
                running_count,
                data,
            } => {
                defmt::write!(
                    fmt,
                    "FileDumpMsg::Packet {{ running_count: {}, data: {:?} }}",
                    running_count,
                    data
                )
            }
            Self::Request {
                requester_device,
                file_type,
                name,
            } => {
                defmt::write!(
                    fmt,
                    "FileDumpMsg::Request {{ requester_device: {:?}, file_type: {:?}, name: {:?} }}",
                    requester_device, file_type, name.as_slice()
                )
            }
        }
    }
}

impl FileDumpMsg {
    pub(crate) fn extend_midi<E>(&self, mut v: impl Write<Error = E>) -> Result<(), E> {
        match self {
            Self::Header {
                sender_device,
                file_type,
                length,
                name,
            } => {
                v.push(01)?;
                v.push(sender_device.to_u8())?;
                file_type.extend_midi(&mut v)?;
                push_u28(*length, &mut v)?;
                v.write(name)
            }
            Self::Packet {
                running_count,
                data,
                ..
            } => {
                v.push(02)?;
                v.push(to_u7(*running_count))?;
                let mut len = data.len().min(112);
                // Add number of extra encoded bytes
                // (/ 7 is -1 of actual number of encoded bytes, but it's sent as length - 1)
                len += len / 7;
                assert!(len < 128);
                v.push(len as u8)?;
                v.write(&Self::encode_data(data))
            }
            Self::Request {
                requester_device,
                file_type,
                name,
            } => {
                v.push(03)?;
                v.push(requester_device.to_u8())?;
                file_type.extend_midi(&mut v)?;
                v.write(name)
            }
        }
    }

    /// Construct a packet of up to 112 (full) bytes.
    /// `num` is the number of this packet.
    pub fn packet(num: u32, data: Vec<u8>) -> Self {
        Self::Packet {
            running_count: (num % 128) as u8,
            data,
        }
    }

    fn encode_data(data: &[u8]) -> Vec<u8> {
        let mut r = Vec::with_capacity(128);
        let mut d = 0; // Data position
        let mut e = 0; // Encoded position
        loop {
            if e >= 128 || d >= data.len() {
                break;
            }
            r.push(0); // First bits
            let mut j = 0;
            loop {
                if j >= 7 || d + j >= data.len() {
                    break;
                }
                r[e] += (data[d + j] >> 7) << (6 - j);
                r.push(data[d + j] & 0b01111111);
                j += 1;
            }

            e += 8;
            d += j;
        }
        r
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::NotImplemented("FileDumpMsg"))
    }
}

/// A four-character file type used by [`FileDumpMsg`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FileType {
    MIDI,
    MIEX,
    ESEQ,
    TEXT,
    BIN,
    MAC,
    Custom([u8; 4]),
}

impl FileType {
    fn extend_midi<E>(&self, mut v: impl Write<Error = E>) -> Result<(), E> {
        match self {
            Self::MIDI => v.write(b"MIDI"),
            Self::MIEX => v.write(b"MIEX"),
            Self::ESEQ => v.write(b"ESEQ"),
            Self::TEXT => v.write(b"TEXT"),
            Self::BIN => v.write(b"BIN "),
            Self::MAC => v.write(b"MAC "),
            Self::Custom(chars) => v.write(chars),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use alloc::vec;

    #[test]
    fn encode_data() {
        assert_eq!(
            FileDumpMsg::encode_data(&[
                0b11111111, 0b10101010, 0b00000000, 0b01010101, 0b11111111, 0b10101010, 0b00000000,
                0b11010101
            ]),
            vec![
                0b01100110, 0b01111111, 0b00101010, 0b00000000, 0b01010101, 0b01111111, 0b00101010,
                0b00000000, 0b01000000, 0b01010101
            ]
        );
    }

    #[test]
    fn serialize_file_dump_packet() {
        let packet_msg = MidiMsg::SystemExclusive {
            msg: SystemExclusiveMsg::UniversalNonRealTime {
                device: DeviceID::AllCall,
                msg: UniversalNonRealTimeMsg::FileDump(FileDumpMsg::packet(
                    129,
                    vec![
                        0b11111111, 0b10101010, 0b00000000, 0b01010101, 0b11111111, 0b10101010,
                        0b00000000, 0b11010101,
                    ],
                )),
            },
        }
        .to_midi();

        assert_eq!(packet_msg.len(), 19);
        assert_eq!(&packet_msg[0..7], &[0xF0, 0x7E, 0x7F, 0x07, 0x02, 0x01, 9]);
        assert_eq!(
            &packet_msg[7..17],
            &[
                0b01100110, 0b01111111, 0b00101010, 0b00000000, 0b01010101, 0b01111111, 0b00101010,
                0b00000000, 0b01000000, 0b01010101
            ]
        );
        assert_eq!(
            packet_msg[17], // Checksum
            checksum(&[
                0x7E, 0x7F, 0x07, 0x02, 0x01, 9, 0b01100110, 0b01111111, 0b00101010, 0b00000000,
                0b01010101, 0b01111111, 0b00101010, 0b00000000, 0b01000000, 0b01010101
            ])
        );
    }

    #[test]
    fn serialize_file_dump_header() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalNonRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalNonRealTimeMsg::FileDump(FileDumpMsg::Header {
                        sender_device: DeviceID::Device(9),
                        file_type: FileType::MIDI,
                        length: 66,
                        name: BString::from("Hello"),
                    }),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7E, 0x7F, // Receiver device
                0x7, 0x1, 0x9, // Sender device
                b"M"[0], b"I"[0], b"D"[0], b"I"[0], 66, // Size LSB
                0x0, 0x0, 0x0, b"H"[0], b"e"[0], b"l"[0], b"l"[0], b"o"[0], 0xF7
            ]
        );
    }
}
