use crate::util::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SampleDumpMsg {
    Header {
        /// 0-16383
        sample_num: u16,
        /// # of significant bits from 8-28
        format: u8,
        /// Sample period (1/sample rate) in nanoseconds, 0-2097151
        period: u32,
        /// Sample length in words, 0-2097151
        length: u32,
        /// Sustain loop start point word number, 0-2097151
        sustain_loop_start: u32,
        /// Sustain loop end point word number, 0-2097151
        sustain_loop_end: u32,
        loop_type: LoopType,
    },
    /// Use `packet` to construct
    Packet {
        /// Running packet count, 0-127. Wraps back to 0
        running_count: u8,
        /// At most 120 7 bit words
        data: Vec<u8>,
    },
    Request {
        sample_num: u16,
    },
    MultipleLoopPoints {
        sample_num: u16,
        /// 0-126. 127 indicates "delete all loops"
        loop_num: LoopNumber,
        loop_type: LoopType,
        /// Loop start address (in samples)
        start_addr: u32,
        /// Loop end address (in samples)
        end_addr: u32,
    },
    LoopPointsRequest {
        sample_num: u16,
        /// 0-126. 127 indicates "request all loops"
        loop_num: LoopNumber,
    },
}

impl SampleDumpMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Header {
                sample_num,
                format,
                period,
                length,
                sustain_loop_start,
                sustain_loop_end,
                loop_type,
            } => {
                push_u14(*sample_num, v);
                v.push((*format).min(28).max(8));
                push_u21(*period, v);
                push_u21(*length, v);
                push_u21(*sustain_loop_start, v);
                push_u21(*sustain_loop_end, v);
                v.push(loop_type.to_u8());
            }
            Self::Packet {
                running_count,
                data,
                ..
            } => {
                let mut p: [u8; 120] = [0; 120];
                for (i, b) in data.iter().enumerate() {
                    if i > 119 {
                        break;
                    }
                    p[i] = to_u7(*b);
                }
                v.push(to_u7(*running_count));
                v.extend_from_slice(&p);
                v.push(0); // Checksum <- Will be written over by `SystemExclusiveMsg.extend_midi`
            }
            Self::Request { sample_num } => {
                push_u14(*sample_num, v);
            }
            Self::MultipleLoopPoints {
                sample_num,
                loop_num,
                loop_type,
                start_addr,
                end_addr,
            } => {
                push_u14(*sample_num, v);
                loop_num.extend_midi(v);
                v.push(loop_type.to_u8());
                push_u21(*start_addr, v);
                push_u21(*end_addr, v);
            }
            Self::LoopPointsRequest {
                sample_num,
                loop_num,
            } => {
                push_u14(*sample_num, v);
                loop_num.extend_midi(v);
            }
        }
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }

    pub fn packet(num: u32, mut data: [u8; 120]) -> Self {
        for d in data.iter_mut() {
            *d = to_u7(*d);
        }

        Self::Packet {
            running_count: (num % 128) as u8,
            data: data.to_vec(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LoopNumber {
    RequestAll,
    DeleteAll,
    Loop(u16),
}

impl LoopNumber {
    fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::RequestAll => {
                v.push(0x7F);
                v.push(0x7F);
            }
            Self::DeleteAll => {
                v.push(0x7F);
                v.push(0x7F);
            }
            Self::Loop(x) => push_u14(*x, v),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LoopType {
    Forward,
    BiDirectional,
    Off,
}

impl LoopType {
    fn to_u8(&self) -> u8 {
        match self {
            Self::Forward => 0x00,
            Self::BiDirectional => 0x01,
            Self::Off => 0x7F,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn serialize_sample_dump_msg() {
        let mut data: [u8; 120] = [0; 120];
        data[0] = 0xFF; // Will become 0x7F;
        let packet_msg = MidiMsg::SystemExclusive {
            msg: SystemExclusiveMsg::UniversalNonRealTime {
                device: DeviceID::AllCall,
                msg: UniversalNonRealTimeMsg::SampleDump(SampleDumpMsg::packet(129, data)),
            },
        }
        .to_midi();

        assert_eq!(packet_msg.len(), 127);
        assert_eq!(&packet_msg[0..6], &[0xF0, 0x7E, 0x7F, 0x02, 0x01, 0x7F]);
        assert_eq!(
            packet_msg[125], // Checksum
            crate::util::checksum(&[0x7E, 0x7F, 0x02, 0x01, 0x7F])
        );
    }
}
