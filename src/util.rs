use super::ParseError;
use alloc::vec::Vec;
use micromath::F32Ext;

#[inline]
pub fn to_u7(x: u8) -> u8 {
    x.min(127)
}

#[inline]
pub fn i_to_u7(x: i8) -> u8 {
    to_u7((x.max(-64) + 64) as u8)
}

#[inline]
pub fn u7_to_i(x: u8) -> i8 {
    x as i8 - 64
}

#[inline]
pub fn bool_from_u7(x: u8) -> Result<bool, ParseError> {
    if x > 127 {
        Err(ParseError::ByteOverflow)
    } else {
        Ok(x >= 0x40)
    }
}

#[inline]
pub fn u8_from_u7(x: u8) -> Result<u8, ParseError> {
    if x > 127 {
        Err(ParseError::ByteOverflow)
    } else {
        Ok(x)
    }
}

#[inline]
pub fn u7_from_midi(m: &[u8]) -> Result<u8, ParseError> {
    if m.len() < 1 {
        Err(ParseError::UnexpectedEnd)
    } else {
        u8_from_u7(m[0])
    }
}

// #[inline]
// pub fn to_i7(x: i8) -> u8 {
//     if x > 63 {
//         0x7f
//     } else if x < -64 {
//         0x40
//     } else {
//         x as u8 & 0b01111111
//     }
// }

#[inline]
pub fn to_u14(x: u16) -> [u8; 2] {
    if x > 16383 {
        [0x7f, 0x7f]
    } else {
        [(x >> 7) as u8, x as u8 & 0b01111111]
    }
}

#[inline]
pub fn i_to_u14(x: i16) -> [u8; 2] {
    to_u14((x.max(-8192) + 8192) as u16)
}

#[inline]
pub fn u14_from_midi(m: &[u8]) -> Result<u16, ParseError> {
    if m.len() < 2 {
        Err(crate::ParseError::UnexpectedEnd)
    } else {
        let (lsb, msb) = (m[0], m[1]);
        if lsb > 127 || msb > 127 {
            Err(ParseError::ByteOverflow)
        } else {
            let mut x = lsb as u16;
            x += (msb as u16) << 7;
            Ok(x)
        }
    }
}

#[inline]
pub fn replace_u14_lsb(msb: u16, lsb: u8) -> u16 {
    (msb & 0b11111110000000) + (lsb as u16)
}

#[inline]
pub fn u14_from_u7s(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 7) + (lsb as u16)
}

#[inline]
pub fn i14_from_u7s(msb: u8, lsb: u8) -> i16 {
    u14_from_u7s(msb, lsb) as i16 - 8192
}

#[inline]
pub fn to_nibble(x: u8) -> [u8; 2] {
    [x >> 4, x & 0b00001111]
}

#[inline]
pub fn push_u7(x: u8, v: &mut Vec<u8>) {
    v.push(to_u7(x));
}

// #[inline]
// pub fn push_i7(x: i8, v: &mut Vec<u8>) {
//     v.push(to_i7(x));
// }

#[inline]
pub fn push_u14(x: u16, v: &mut Vec<u8>) {
    let [msb, lsb] = to_u14(x);
    v.push(lsb);
    v.push(msb);
}

/// Given a frequency in Hertz, returns a floating point midi note number with 1.0 = 100 cents
pub fn freq_to_midi_note_float(freq: f32) -> f32 {
    12.0 * F32Ext::log2(freq / 440.0) + 69.0
}

/// Given a floating point midi note number, return the frequency in Hertz
pub fn midi_note_float_to_freq(note: f32) -> f32 {
    F32Ext::powf(2.0, (note - 69.0) / 12.0) * 440.0
}

/// Given a midi note number and additional cents, return the frequency
pub fn midi_note_cents_to_freq(note: u8, cents: f32) -> f32 {
    midi_note_float_to_freq(note as f32 + cents / 100.0)
}

/// Given a frequency in Hertz, returns (midi_note_number, additional cents from semitone)
pub fn freq_to_midi_note_cents(freq: f32) -> (u8, f32) {
    let semitone = freq_to_midi_note_float(freq);
    (semitone as u8, F32Ext::fract(semitone) * 100.0)
}

#[cfg(feature = "sysex")]
mod sysex_util {
    use alloc::vec::Vec;

    #[inline]
    pub fn push_i14(x: i16, v: &mut Vec<u8>) {
        let [msb, lsb] = to_i14(x);
        v.push(lsb);
        v.push(msb);
    }

    #[inline]
    pub fn push_u21(x: u32, v: &mut Vec<u8>) {
        let [msb, b, lsb] = to_u21(x);
        v.push(lsb);
        v.push(b);
        v.push(msb);
    }

    #[inline]
    pub fn push_u28(x: u32, v: &mut Vec<u8>) {
        let [mmsb, msb, lsb, llsb] = to_u28(x);
        v.push(llsb);
        v.push(lsb);
        v.push(msb);
        v.push(mmsb);
    }

    #[inline]
    pub fn push_u35(x: u64, v: &mut Vec<u8>) {
        let [msb, b2, b3, b4, lsb] = to_u35(x);
        v.push(lsb);
        v.push(b4);
        v.push(b3);
        v.push(b2);
        v.push(msb);
    }

    pub fn checksum(bytes: &[u8]) -> u8 {
        let mut sum: u8 = 0;
        for b in bytes.iter() {
            sum ^= b;
        }
        sum
    }

    /// Takes a positive value between 0.0 and 100.0 and fits it into the u14 range
    /// 1 = 0.0061 cents
    pub fn cents_to_u14(cents: f32) -> u16 {
        let cents = cents.max(0.0).min(100.0);
        super::F32Ext::round(cents / 100.0 * (0b11111111111111 as f32)) as u16
    }

    #[inline]
    pub fn to_i14(x: i16) -> [u8; 2] {
        if x > 8191 {
            [0x3f, 0x7f]
        } else if x < -8191 {
            [0x40, 0x00]
        } else {
            [(x >> 7) as u8 & 0b01111111, x as u8 & 0b01111111]
        }
    }

    #[inline]
    pub fn to_u21(x: u32) -> [u8; 3] {
        if x > 2097151 {
            [0x7f, 0x7f, 0x7f]
        } else {
            [
                (x >> 14) as u8,
                (x >> 7) as u8 & 0b01111111,
                x as u8 & 0b01111111,
            ]
        }
    }

    #[inline]
    pub fn to_u28(x: u32) -> [u8; 4] {
        if x > 2684354561 {
            [0x7f, 0x7f, 0x7f, 0x7f]
        } else {
            [
                (x >> 21) as u8,
                (x >> 14) as u8 & 0b01111111,
                (x >> 7) as u8 & 0b01111111,
                x as u8 & 0b01111111,
            ]
        }
    }

    #[inline]
    pub fn to_u35(x: u64) -> [u8; 5] {
        if x > 34359738367 {
            [0x7f, 0x7f, 0x7f, 0x7f, 0x7f]
        } else {
            [
                (x >> 28) as u8,
                (x >> 21) as u8 & 0b01111111,
                (x >> 14) as u8 & 0b01111111,
                (x >> 7) as u8 & 0b01111111,
                x as u8 & 0b01111111,
            ]
        }
    }
}

#[cfg(feature = "sysex")]
pub use sysex_util::*;

#[cfg(feature = "file")]
mod file_util {
    use super::ParseError;
    use alloc::vec::Vec;
    use core::convert::TryInto;

    #[inline]
    pub fn push_u16(x: u16, v: &mut Vec<u8>) {
        let [b1, b2] = x.to_be_bytes();
        v.push(b1);
        v.push(b2);
    }

    #[inline]
    pub fn push_u32(x: u32, v: &mut Vec<u8>) {
        let [b1, b2, b3, b4] = x.to_be_bytes();
        v.push(b1);
        v.push(b2);
        v.push(b3);
        v.push(b4);
    }

    // Variable length quanity
    pub fn push_vlq(x: u32, v: &mut Vec<u8>) {
        if x < 0x00000080 {
            v.push(x as u8 & 0b01111111);
        } else if x < 0x00004000 {
            v.push(((x >> 7) as u8 & 0b01111111) + 0b10000000);
            v.push(x as u8 & 0b01111111);
        } else if x < 0x00200000 {
            v.push(((x >> 14) as u8 & 0b01111111) + 0b10000000);
            v.push(((x >> 7) as u8 & 0b01111111) + 0b10000000);
            v.push(x as u8 & 0b01111111);
        } else if x <= 0x0FFFFFFF {
            v.push(((x >> 21) as u8 & 0b01111111) + 0b10000000);
            v.push(((x >> 14) as u8 & 0b01111111) + 0b10000000);
            v.push(((x >> 7) as u8 & 0b01111111) + 0b10000000);
            v.push(x as u8 & 0b01111111);
        } else {
            panic!("Cannot use such a large number as a variable quantity")
        }
    }

    /*
    #[inline]
        pub fn u16_from_midi(m: &[u8]) -> Result<u16, ParseError> {
            if m.len() < 2 {
                Err(crate::ParseError::UnexpectedEnd)
            } else {
                Ok(u16::from_be_bytes(m[0..2].try_into().unwrap()))
            }
        }
     */

    #[inline]
    pub fn u32_from_midi(m: &[u8]) -> Result<u32, ParseError> {
        if m.len() < 4 {
            Err(crate::ParseError::UnexpectedEnd)
        } else {
            Ok(u32::from_be_bytes(m[0..4].try_into().unwrap()))
        }
    }

    pub fn read_vlq(data: &[u8]) -> Result<(u32, usize), ParseError> {
        let mut result: u32 = 0;
        let mut bytes_read = 0;

        for &byte in data {
            result = (result << 7) | (byte & 0b01111111) as u32;
            bytes_read += 1;

            if byte & 0b10000000 == 0 {
                // If the MSB is not set, this is the last byte of the VLQ
                return Ok((result, bytes_read));
            }

            // Check if we've read too many bytes for a VLQ
            if bytes_read >= 4 {
                return Err(ParseError::VlqOverflow);
            }
        }

        // If we've exhausted the slice without finding the end of VLQ
        Err(ParseError::UnexpectedEnd)
    }
}

#[cfg(feature = "file")]
pub use file_util::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_u7() {
        assert_eq!(to_u7(0xff), 127); // Overflow is treated as max value
        assert_eq!(to_u7(0x77), 0x77);
        assert_eq!(to_u7(0x00), 0x00);
        assert_eq!(to_u7(0x7f), 127);
    }

    #[test]
    fn test_to_u14() {
        assert_eq!(to_u14(0xff), [1, 127]);
        assert_eq!(to_u14(0xff00), [127, 127]); // Overflow is treated as max value
        assert_eq!(to_u14(0x00), [0, 0]);
        assert_eq!(to_u14(0xfff), [0x1f, 127]);
        assert_eq!(to_u14(1000), [0x07, 0x68]);
    }

    #[test]
    fn test_i_to_u() {
        assert_eq!(i_to_u7(63), 127);
        assert_eq!(i_to_u7(0), 64);
        assert_eq!(i_to_u7(-64), 0);

        assert_eq!(i_to_u14(0), to_u14(8192));
        assert_eq!(i_to_u14(-8192), to_u14(0));
        assert_eq!(i_to_u14(8191), to_u14(16383));
    }

    #[test]
    fn test_u_to_i() {
        assert_eq!(u7_to_i(127), 63);
        assert_eq!(u7_to_i(64), 0);
        assert_eq!(u7_to_i(0), -64);

        assert_eq!(i14_from_u7s(to_u14(8192)[0], to_u14(8192)[1]), 0);
        assert_eq!(i14_from_u7s(0, 0), -8192);
        assert_eq!(i14_from_u7s(to_u14(16383)[0], to_u14(16383)[1]), 8191);
    }

    #[test]
    fn test_midi_note_float_to_freq() {
        assert!((midi_note_float_to_freq(67.0) - 392.0).abs() <= 0.01);
    }

    #[test]
    #[cfg(feature = "sysex")]
    fn test_to_i14() {
        assert_eq!(to_i14(0xff), [0x01, 0x7f]);
        assert_eq!(to_i14(0x6f00), [0x3f, 0x7f]); // Overflow is treated as max value
        assert_eq!(to_i14(0x00), [0, 0]);
        assert_eq!(to_i14(0xfff), [0x1f, 0x7f]);
        assert_eq!(to_i14(1000), [0x07, 0x68]);
        assert_eq!(to_i14(-10000), [0x40, 0x00]); // Min overflow is treated as min value
        assert_eq!(to_i14(-8192), [0x40, 0x00]);
        assert_eq!(to_i14(-8191), [0x40, 0x01]);
        assert_eq!(to_i14(-1), [0x7f, 0x7f]);
    }

    #[test]
    #[cfg(feature = "sysex")]
    fn test_to_u21() {
        assert_eq!(to_u21(0xff), [0, 1, 127]);
        assert_eq!(to_u21(0xff00), [3, 126, 0]);
        assert_eq!(to_u21(0xffff00), [127, 127, 127]); // Overflow is treated as max value
        assert_eq!(to_u21(0x00), [0, 0, 0]);
        assert_eq!(to_u21(0xfff), [0, 0x1f, 127]);
        assert_eq!(
            to_u21(0b1000011010100000),
            [0b00000010, 0b00001101, 0b00100000]
        );
    }

    #[test]
    #[cfg(feature = "sysex")]
    fn text_checksum() {
        assert_eq!(checksum(&[0b11110000, 0b00001111, 0b10101010]), 0b01010101);
        assert_eq!(
            checksum(&[0x41, 0x4D, 0x02, 0x41, 0x21, 0x04, 0x02, 0x02]),
            0x6A
        )
    }

    #[cfg(feature = "sysex")]
    fn freq_to_midi_note_u14(freq: f32) -> (u8, u16) {
        let (n, c) = crate::freq_to_midi_note_cents(freq);
        (n, cents_to_u14(c))
    }

    #[test]
    #[cfg(feature = "sysex")]
    fn test_freq_to_midi_note() {
        // The test data below is taken from the "Frequency data format" section (page 48)
        // of The MIDI 1.0 Detailed Specification 4.2.1.

        // This crate uses micromath for fast, no_std friendly approximations
        // for math functions like powf and ln2. The tests below verify
        // that these approximations are reasonable.

        // Frequency      : 8.1758 Hz
        // Expected bytes : 00 00 00
        // Actual bytes   : 00 00 0f
        // Error          : 0.0061 * 15 = 0.0915 cents
        assert_eq!(freq_to_midi_note_u14(8.1758), (0x00, 0x0f));

        // Frequency      : 8.662 Hz
        // Expected bytes : 01 00 00
        // Actual bytes   : 01 00 11
        // Error          : 0.0061 * 17 = 0.1037 cents
        assert_eq!(freq_to_midi_note_u14(8.662), (0x01, 0x11));

        // Frequency      : 261.6256 Hz
        // Expected bytes : 3c 00 00
        // Actual bytes   : 3c 00 0f
        // Error          : 0.0061 * 15 = 0.0915 cents
        assert_eq!(freq_to_midi_note_u14(261.6256), (0x3C, 0x0f));

        // This happens to be an exact match since the refrence frequency
        // for the approximation is 440.000 (the spec example data says
        // note number 0x43, which seems to be wrong).
        assert_eq!(freq_to_midi_note_u14(440.0000), (0x45, 0x00));

        // Frequency      : 8372.0190 Hz
        // Expected bytes : 78 00 00
        // Actual bytes   : 78 00 01
        // Error          : 0.0061 cents
        assert_eq!(freq_to_midi_note_u14(8372.0190), (0x78, 0x01));

        // Frequency      : 12543.8800 Hz
        // Expected bytes : 7f 00 00
        // Actual bytes   : 7f 00 02
        // Error          : 0.0061 * 2 = 0.0122 cents
        assert_eq!(freq_to_midi_note_u14(12543.8800), (0x7F, 0x02));
    }

    #[test]
    #[cfg(feature = "file")]
    fn test_vlq() {
        fn test(x: u32, expected_len: usize) {
            let mut v = Vec::new();
            push_vlq(x, &mut v);
            let (y, len) = read_vlq(&v).unwrap();
            assert_eq!(
                x, y,
                "Got {} after converting {} to and from a variable-length quantity",
                y, x
            );
            assert_eq!(
                len, expected_len,
                "Expected a variable-length quantity of length {} but got {}",
                expected_len, len
            );
        }
        test(0, 1);
        test(0x40, 1);
        test(0x7F, 1);
        test(0x80, 2);
        test(0x2000, 2);
        test(0x3FFF, 2);
        test(0x4000, 3);
        test(0x100000, 3);
        test(0x1FFFFF, 3);
        test(0x200000, 4);
        test(0x8000000, 4);
        test(0xFFFFFFF, 4);

        assert_eq!(
            read_vlq(&[0x80, 0x80, 0x80, 0x80]),
            Err(ParseError::VlqOverflow)
        );
    }
}
