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
