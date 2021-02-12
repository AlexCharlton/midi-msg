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
        self.into()
    }
}

impl From<&SystemRealTimeMsg> for Vec<u8> {
    fn from(m: &SystemRealTimeMsg) -> Vec<u8> {
        match m {
            SystemRealTimeMsg::TimingClock => vec![0xF8],
            SystemRealTimeMsg::Start => vec![0xFA],
            SystemRealTimeMsg::Continue => vec![0xFB],
            SystemRealTimeMsg::Stop => vec![0xFC],
            SystemRealTimeMsg::ActiveSensing => vec![0xFE],
            SystemRealTimeMsg::SystemReset => vec![0xFF],
        }
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
