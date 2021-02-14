use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeCode {
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

impl TimeCode {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UserBits {
    /// Full bytes can be used here. Sent such that the first is considered
    /// the "most significant" value
    bytes: (u8, u8, u8, u8),
    /// SMPTE time code bit 43 (EBU bit 27)
    flag1: bool,
    /// SMPTE time code bit 59 (EBU bit 43)
    flag2: bool,
}

impl UserBits {
    pub fn to_nibbles(&self) -> [u8; 9] {
        let [uh, ug] = to_nibble(self.bytes.3);
        let [uf, ue] = to_nibble(self.bytes.2);
        let [ud, uc] = to_nibble(self.bytes.1);
        let [ub, ua] = to_nibble(self.bytes.0);
        let mut flags: u8 = 0;
        if self.flag1 {
            flags += 1;
        }
        if self.flag2 {
            flags += 2;
        }
        [ua, ub, uc, ud, ue, uf, ug, uh, flags]
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeCodeMsg {
    //TODO
}

impl TimeCodeMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        // TODO
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TimeCodeCueingMsg {
    //TODO
}

impl TimeCodeCueingMsg {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        // TODO
    }

    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn serialize_time_code_msg() {
        // TODO
        // assert_eq!(
        //     MidiMsg::ChannelVoice {
        //         channel: Channel::Ch1,
        //         msg: ChannelVoiceMsg::NoteOn {
        //             note: 0x88,
        //             velocity: 0xff
        //         }
        //     }
        //     .to_midi(),
        //     vec![0x90, 0x7f, 127]
        // );
    }
}
