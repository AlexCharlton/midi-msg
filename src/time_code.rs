use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Frame {
    /// 0-29
    pub frame: u8,
    /// 0-59
    pub seconds: u8,
    /// 0-59
    pub minutes: u8,
    /// 0-23
    pub hours: u8,
    pub code_type: TimeCodeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeCodeType {
    FPS24 = 0,
    FPS25 = 1,
    DF30 = 2,
    NDF30 = 3,
}

impl Default for TimeCodeType {
    fn default() -> Self {
        Self::NDF30
    }
}

impl Frame {
    // Return the four byte representation of the frame: [frame, seconds, minutes, timecode + hours]
    pub fn to_bytes(self) -> [u8; 4] {
        [
            self.frame.min(29),
            self.seconds.min(59),
            self.minutes.min(59),
            self.hours.min(23) + ((self.code_type as u8) << 5),
        ]
    }

    // Return an 8 byte, Quarter Frame representation of the Frame
    pub fn to_nibbles(self) -> [u8; 8] {
        let [frame, seconds, minutes, codehour] = self.to_bytes();

        let [frame_msb, frame_lsb] = to_nibble(frame);
        let [seconds_msb, seconds_lsb] = to_nibble(seconds);
        let [minutes_msb, minutes_lsb] = to_nibble(minutes);
        let [codehour_msb, codehour_lsb] = to_nibble(codehour);
        [
            (0 << 4) + frame_lsb,
            (1 << 4) + frame_msb,
            (2 << 4) + seconds_lsb,
            (3 << 4) + seconds_msb,
            (4 << 4) + minutes_lsb,
            (5 << 4) + minutes_msb,
            (6 << 4) + codehour_lsb,
            (7 << 4) + codehour_msb,
        ]
    }
}
