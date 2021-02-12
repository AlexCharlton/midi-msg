#[inline]
pub fn to_u7(x: u8) -> u8 {
    x.min(127)
}

#[inline]
pub fn to_u14(x: u16) -> [u8; 2] {
    [to_u7((x >> 7) as u8), to_u7(x as u8 & 0b01111111)]
}

#[inline]
pub fn to_nibble(x: u8) -> [u8; 2] {
    [x >> 4, x & 0b00001111]
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
        assert_eq!(to_u14(0xffff), [127, 127]); // Overflow is treated as max value
        assert_eq!(to_u14(0x00), [0, 0]);
        assert_eq!(to_u14(0xfff), [0x1f, 127]);
        assert_eq!(to_u14(1000), [0x07, 0x68]);
    }
}
