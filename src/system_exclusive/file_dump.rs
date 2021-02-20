use super::DeviceID;
use crate::util::*;
use ascii::{AsciiChar, AsciiString};

#[derive(Debug, Clone, PartialEq)]
pub enum FileDumpMsg {
    Header {
        sender_device: DeviceID,
        file_type: FileType,
        /// Actual (un-encoded) file length, 28 bits (0-2684354561)
        length: u32,
        name: AsciiString,
    },
    /// Use `packet` to construct
    Packet {
        /// Running packet count, 0-127. Wraps back to 0
        running_count: u8,
        /// At most 112 bytes (full 8 bits may be used)
        data: Vec<u8>,
    },
    Request {
        requester_device: DeviceID,
        file_type: FileType,
        name: AsciiString,
    },
}

impl FileDumpMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Header {
                sender_device,
                file_type,
                length,
                name,
            } => {
                v.push(01);
                v.push(sender_device.to_u8());
                file_type.extend_midi(v);
                push_u28(*length, v);
                v.extend_from_slice(name.as_bytes());
            }
            Self::Packet {
                running_count,
                data,
                ..
            } => {
                v.push(02);
                v.push(to_u7(*running_count));
                let mut len = data.len().min(112);
                // Add number of extra encoded bytes
                // (/ 7 is -1 of actual number of encoded bytes, but it's sent as length - 1)
                len += len / 7;
                assert!(len < 128);
                v.push(len as u8);
                v.extend(Self::encode_data(data));
                v.push(0); // Checksum <- Will be written over by `SystemExclusiveMsg.extend_midi`
            }
            Self::Request {
                requester_device,
                file_type,
                name,
            } => {
                v.push(03);
                v.push(requester_device.to_u8());
                file_type.extend_midi(v);
                v.extend_from_slice(name.as_bytes());
            }
        }
    }

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

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileType {
    MIDI,
    MIEX,
    ESEQ,
    TEXT,
    BIN,
    MAC,
    Custom([AsciiChar; 4]),
}

impl FileType {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::MIDI => {
                v.push(AsciiChar::M.as_byte());
                v.push(AsciiChar::I.as_byte());
                v.push(AsciiChar::D.as_byte());
                v.push(AsciiChar::I.as_byte());
            }
            Self::MIEX => {
                v.push(AsciiChar::M.as_byte());
                v.push(AsciiChar::I.as_byte());
                v.push(AsciiChar::E.as_byte());
                v.push(AsciiChar::X.as_byte());
            }
            Self::ESEQ => {
                v.push(AsciiChar::E.as_byte());
                v.push(AsciiChar::S.as_byte());
                v.push(AsciiChar::E.as_byte());
                v.push(AsciiChar::Q.as_byte());
            }
            Self::TEXT => {
                v.push(AsciiChar::T.as_byte());
                v.push(AsciiChar::E.as_byte());
                v.push(AsciiChar::X.as_byte());
                v.push(AsciiChar::T.as_byte());
            }
            Self::BIN => {
                v.push(AsciiChar::B.as_byte());
                v.push(AsciiChar::I.as_byte());
                v.push(AsciiChar::N.as_byte());
                v.push(AsciiChar::Space.as_byte());
            }
            Self::MAC => {
                v.push(AsciiChar::M.as_byte());
                v.push(AsciiChar::A.as_byte());
                v.push(AsciiChar::C.as_byte());
                v.push(AsciiChar::Space.as_byte());
            }
            Self::Custom(chars) => {
                v.push(chars[0].as_byte());
                v.push(chars[1].as_byte());
                v.push(chars[2].as_byte());
                v.push(chars[3].as_byte());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

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
            crate::util::checksum(&[
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
                        name: AsciiString::from_ascii("Hello").unwrap(),
                    }),
                },
            }
            .to_midi(),
            vec![
                0xF0,
                0x7E,
                0x7F, // Receiver device
                07,
                01,
                9, // Sender device
                AsciiChar::M.as_byte(),
                AsciiChar::I.as_byte(),
                AsciiChar::D.as_byte(),
                AsciiChar::I.as_byte(),
                66, // Size LSB
                0,
                0,
                0,
                AsciiChar::H.as_byte(),
                AsciiChar::e.as_byte(),
                AsciiChar::l.as_byte(),
                AsciiChar::l.as_byte(),
                AsciiChar::o.as_byte(),
                0xF7
            ]
        );
    }
}
