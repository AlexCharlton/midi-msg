use micromath::F32Ext;

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
  F32Ext::round(cents / 100.0 * (0b11111111111111 as f32)) as u16
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let (n, c) = crate::freq_to_midi_note_cents(freq);
        (n, cents_to_u14(c))
    }

    #[test]
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
}
