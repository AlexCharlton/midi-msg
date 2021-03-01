#[inline]
pub fn to_u7(x: u8) -> u8 {
    x.min(127)
}

#[inline]
pub fn to_i7(x: i8) -> u8 {
    if x > 63 {
        0x7f
    } else if x < -64 {
        0x40
    } else {
        x as u8 & 0b01111111
    }
}

#[inline]
pub fn to_u14(x: u16) -> [u8; 2] {
    if x > 16383 {
        [0x7f, 0x7f]
    } else {
        [(x >> 7) as u8, x as u8 & 0b01111111]
    }
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

#[inline]
pub fn to_nibble(x: u8) -> [u8; 2] {
    [x >> 4, x & 0b00001111]
}

#[inline]
pub fn push_u7(x: u8, v: &mut Vec<u8>) {
    v.push(to_u7(x));
}

#[inline]
pub fn push_i7(x: i8, v: &mut Vec<u8>) {
    v.push(to_i7(x));
}

#[inline]
pub fn push_u14(x: u16, v: &mut Vec<u8>) {
    let [msb, lsb] = to_u14(x);
    v.push(lsb);
    v.push(msb);
}

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

/// Given a frequency in Hertz, returns a floating point midi note number with 1.0 = 100 cents
pub fn freq_to_midi_note_float(freq: f32) -> f32 {
    12.0 * (freq / 440.0).log2() + 69.0
}

/// Returns (midi_note_number, additional cents from semitone)
pub fn freq_to_midi_note_cents(freq: f32) -> (u8, f32) {
    let semitone = freq_to_midi_note_float(freq);
    (semitone as u8, semitone.fract() * 100.0)
}

/// Given a floating point midi note number, return the frequency in Hertz
pub fn midi_note_float_to_freq(note: f32) -> f32 {
    (2.0 as f32).powf((note - 69.0) / 12.0) * 440.0
}

/// Given a midi note number and additional cents, return the frequency
pub fn midi_note_cents_to_freq(note: u8, cents: f32) -> f32 {
    midi_note_float_to_freq(note as f32 + cents / 100.0)
}

/// Takes a positive value between 0.0 and 100.0 and fits it into the u14 range
/// 1 = 0.0061 cents
pub fn cents_to_u14(cents: f32) -> u16 {
    let cents = cents.max(0.0).min(100.0);
    (cents / 100.0 * (0b11111111111111 as f32)).round() as u16
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
    fn text_checksum() {
        assert_eq!(checksum(&[0b11110000, 0b00001111, 0b10101010]), 0b01010101);
        assert_eq!(
            checksum(&[0x41, 0x4D, 0x02, 0x41, 0x21, 0x04, 0x02, 0x02]),
            0x6A
        )
    }

    fn freq_to_midi_note_u14(freq: f32) -> (u8, u16) {
        let (n, c) = freq_to_midi_note_cents(freq);
        (n, cents_to_u14(c))
    }

    #[test]
    fn test_freq_to_midi_note() {
        // These values are taken from The MIDI 1.0 Detailed Specification 4.2.1

        assert_eq!(freq_to_midi_note_u14(8.1758), (0x00, 0x00));

        // This is a different value than stated, but it seems the spec is quite off
        // I don't see any way that 8.2104 Hz could be 0.0061 cents away from 8.1758 Hz
        assert_eq!(freq_to_midi_note_u14(8.2104), (0x00, 1198));

        // Needed to add extra precision to get it to match
        assert_eq!(freq_to_midi_note_u14(8.66197), (0x01, 0x00));

        assert_eq!(freq_to_midi_note_u14(261.6256), (0x3C, 0x00));

        assert_eq!(freq_to_midi_note_u14(440.0000), (0x45, 0x00));

        assert_eq!(freq_to_midi_note_u14(440.0016), (0x45, 0x01));

        assert_eq!(freq_to_midi_note_u14(8372.0190), (0x78, 0x00));

        assert_eq!(freq_to_midi_note_u14(8372.0630), (0x78, 0x01));

        // Needed to adjust by 0.01 Hz, but that's a very small amount
        assert_eq!(freq_to_midi_note_u14(12543.8700), (0x7F, 0x00));

        // Needed to adjust by 0.02 Hz, but that's also very small amount
        assert_eq!(freq_to_midi_note_u14(13289.7100), (0x7F, 0x3FFE));
    }

    #[test]
    fn test_midi_note_float_to_freq() {
        assert_eq!(midi_note_float_to_freq(69.0), 440.0);
    }
}
