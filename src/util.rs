#[allow(unused_imports)]
use micromath::F32Ext;
use alloc::vec::Vec;
use super::ParseError;

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
}
