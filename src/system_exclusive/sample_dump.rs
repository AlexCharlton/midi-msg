use alloc::vec::Vec;
use alloc::format;
use crate::parse_error::*;
use crate::util::*;
use ascii::AsciiString;

/// Used to request and transmit sampler data.
/// Used by [`UniversalNonRealTimeMsg::SampleDump`](crate::UniversalNonRealTimeMsg::SampleDump).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SampleDumpMsg {
    /// Request that the receiver send the given sample.
    Request {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
    },
    /// The first message in a sample dump, used to describe the data contained in the following packets.
    Header {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        /// # of significant bits from 8-28.
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
    /// A single packet of sample data.
    ///
    /// Use [`SampleDumpMsg::packet`] to construct.
    Packet {
        /// Running packet count, 0-127. Wraps back to 0
        running_count: u8,
        /// At most 120 7 bit words
        data: Vec<u8>,
    },
    /// Request that the receiver return data about the loop points for a given sample.
    LoopPointsRequest {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        loop_num: LoopNumber,
    },
    /// Used to send additional loop points for a given sample.
    LoopPointTransmission {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        loop_num: LoopNumber,
        loop_type: LoopType,
        /// Loop start address (in samples)
        start_addr: u32,
        /// Loop end address (in samples)
        end_addr: u32,
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
                v.push(*loop_type as u8);
            }
            Self::Packet {
                running_count,
                data,
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
            Self::LoopPointTransmission {
                sample_num,
                loop_num,
                loop_type,
                start_addr,
                end_addr,
            } => {
                push_u14(*sample_num, v);
                loop_num.extend_midi(v);
                v.push(*loop_type as u8);
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

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }

    /// Construct a packet of exactly 120 7-bit "bytes".
    /// `num` is the number of this packet.
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

/// What loop a [`SampleDumpMsg`] or [`ExtendedSampleDumpMsg`] is referring to.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LoopNumber {
    /// A loop with the given ID, 0-16382.
    Loop(u16),
    /// Used by [`SampleDumpMsg::LoopPointsRequest`] to request all loops.
    RequestAll,
    /// Used by [`SampleDumpMsg::LoopPointTransmission`] to indicate that all loops should be deleted.
    DeleteAll,
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

/// The type of loop being described by a [`SampleDumpMsg`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LoopType {
    /// Forward only
    Forward = 0,
    /// Backward forward
    BiDirectional = 1,
    /// Do not loop
    Off = 127,
}

/// The extended sample dump messages described in CA-019, used to allow for longer, named samples.
/// Used by [`UniversalNonRealTimeMsg::SampleDump`](crate::UniversalNonRealTimeMsg::SampleDump).
#[derive(Debug, Clone, PartialEq)]
pub enum ExtendedSampleDumpMsg {
    Header {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        /// # of significant bits from 8-28
        format: u8,
        /// Sample rate in Hz. The f64 is used to approximate the two 28bit fixed point sent over the wire.
        sample_rate: f64,
        /// Sample length in words, 0-34359738368
        length: u64,
        /// Sustain loop start point word number, 0-34359738367
        sustain_loop_start: u64,
        /// Sustain loop end point word number, 0-34359738367
        sustain_loop_end: u64,
        loop_type: ExtendedLoopType,
        /// Number of audio channels, 0-127
        num_channels: u8,
    },
    /// Request the given sample's name.
    SampleNameRequest {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
    },
    /// Describe the name of a given sample.
    SampleName {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        /// An up to 127 character name.
        name: AsciiString,
    },
    /// Request that the receiver return data about the loop points for a given sample.
    LoopPointsRequest {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        loop_num: LoopNumber,
    },
    /// Used to send additional loop points for a given sample.
    LoopPointTransmission {
        /// The ID of the sample, between 0-16383.
        sample_num: u16,
        loop_num: LoopNumber,
        loop_type: ExtendedLoopType,
        /// Loop start address (in samples)
        start_addr: u64,
        /// Loop end address (in samples)
        end_addr: u64,
    },
}

impl ExtendedSampleDumpMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Header {
                sample_num,
                format,
                sample_rate,
                length,
                sustain_loop_start,
                sustain_loop_end,
                loop_type,
                num_channels,
            } => {
                push_u14(*sample_num, v);
                v.push((*format).min(28).max(8));
                let sample_rate = sample_rate.max(0.0);
                let sample_rate_integer = (sample_rate as u64) as f64; // for lack of no_std f64 floor
                push_u28(sample_rate_integer as u32, v);
                push_u28(
                    ((sample_rate - sample_rate_integer) * ((1 << 28) as f64)) as u32,
                    v,
                );
                push_u35((*length).min(34359738368), v);
                push_u35((*sustain_loop_start).min(34359738367), v);
                push_u35((*sustain_loop_end).min(34359738367), v);
                v.push(*loop_type as u8);
                push_u7(*num_channels, v);
            }
            Self::LoopPointTransmission {
                sample_num,
                loop_num,
                loop_type,
                start_addr,
                end_addr,
            } => {
                push_u14(*sample_num, v);
                loop_num.extend_midi(v);
                v.push(*loop_type as u8);
                push_u35(*start_addr, v);
                push_u35(*end_addr, v);
            }
            Self::LoopPointsRequest {
                sample_num,
                loop_num,
            } => {
                push_u14(*sample_num, v);
                loop_num.extend_midi(v);
            }
            Self::SampleName { sample_num, name } => {
                push_u14(*sample_num, v);
                v.push(0); // Language tag length (0 is the only allowable value)
                let len = name.len().min(127);
                v.push(len as u8);
                v.extend_from_slice(&name.as_bytes()[0..len]);
            }
            Self::SampleNameRequest { sample_num } => {
                push_u14(*sample_num, v);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// The type of loop being described by a [`SampleDumpMsg`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExtendedLoopType {
    /// A forward, unidirectional loop
    Forward = 0x00,
    /// Loop starts playing forward, gets to end and plays backward, repeating
    BiDirectional = 0x01,
    /// A unidirectional loop, upon key release the rest of the sample is played
    ForwardRelease = 0x02,
    /// A bidirectional loop, upon key release the rest of the sample is played
    BiDirectionalRelease = 0x03,
    /// A backward, unidirectional loop
    Backward = 0x40,
    /// Like BiDirectional, but starts playing in reverse
    BackwardBiDirectional = 0x41,
    /// A backward unidirectional loop, upon key release the rest of the sample after the loop is played backwards
    BackwardRelease = 0x42,
    /// A bidirectional loop, starting from the end playing backward, upon key release the rest of the sample after the loop is played backwards
    BackwardBiDirectionalRelease = 0x43,
    /// Backwards one-shot, no looping
    BackwardOneShot = 0x7E,
    /// Forwards one-shot, no looping
    OneShot = 0x7F,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

    #[test]
    fn serialize_sample_dump_msg() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalNonRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalNonRealTimeMsg::ExtendedSampleDump(
                        ExtendedSampleDumpMsg::Header {
                            sample_num: 5,
                            format: 8,
                            sample_rate: 4000.5,
                            length: 2u64.pow(30),
                            sustain_loop_start: 2u64.pow(10),
                            sustain_loop_end: 2u64.pow(20),
                            loop_type: ExtendedLoopType::BiDirectionalRelease,
                            num_channels: 2
                        }
                    ),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7E, 0x7F, // All call
                0x05, 0x05, // ExtendedSampleDump header
                05, 00, // Sample number
                8,  // format,
                0b0100000, 0b0011111, 0, 0, // 4000 LSB first
                0, 0, 0, 0x40, // 0.5 LSB first
                0, 0, 0, 0, 0b0000100, // Length
                0, 0b0001000, 0, 0, 0, // Sustain loop start
                0, 0, 0b1000000, 0, 0,    // Sustain loop end
                0x03, // Loop type
                2,    // Num channels
                0xF7
            ]
        );
    }
}
