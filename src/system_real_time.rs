#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemRealTimeMsg {
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
}

impl SystemRealTimeMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            SystemRealTimeMsg::TimingClock => v.push(0xF8),
            SystemRealTimeMsg::Start => v.push(0xFA),
            SystemRealTimeMsg::Continue => v.push(0xFB),
            SystemRealTimeMsg::Stop => v.push(0xFC),
            SystemRealTimeMsg::ActiveSensing => v.push(0xFE),
            SystemRealTimeMsg::SystemReset => v.push(0xFF),
        }
    }
}

impl From<&SystemRealTimeMsg> for Vec<u8> {
    fn from(m: &SystemRealTimeMsg) -> Vec<u8> {
        m.to_midi()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn serialize_system_real_time_msg() {
        assert_eq!(
            MidiMsg::SystemRealTime {
                msg: SystemRealTimeMsg::TimingClock
            }
            .to_midi(),
            vec![0xF8]
        );
    }
}
