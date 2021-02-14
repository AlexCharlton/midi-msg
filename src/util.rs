#[inline]
pub fn to_u7(x: u8) -> u8 {
    x.min(127)
}

#[inline]
pub fn to_u14(x: u16) -> [u8; 2] {
    if x > 16383 {
        [0x7f, 0x7f]
    } else {
        [to_u7((x >> 7) as u8), to_u7(x as u8 & 0b01111111)]
    }
}

#[inline]
pub fn to_u21(x: u32) -> [u8; 3] {
    if x > 2097151 {
        [0x7f, 0x7f, 0x7f]
    } else {
        [
            to_u7((x >> 14) as u8 & 0b01111111),
            to_u7((x >> 7) as u8),
            to_u7(x as u8 & 0b01111111),
        ]
    }
}

#[inline]
pub fn to_nibble(x: u8) -> [u8; 2] {
    [x >> 4, x & 0b00001111]
}

#[inline]
pub fn push_u14(x: u16, v: &mut Vec<u8>) {
    let [msb, lsb] = to_u14(x);
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

pub fn checksum(bytes: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for b in bytes.iter() {
        sum ^= b;
    }
    sum
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
    fn test_to_u21() {
        assert_eq!(to_u21(0xff), [0, 1, 127]);
        assert_eq!(to_u21(0xff00), [3, 127, 0]);
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
}
