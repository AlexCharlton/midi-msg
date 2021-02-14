use super::time_code::*;
use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemCommonMsg {
    TimeCodeQuarterFrame1(TimeCode),
    TimeCodeQuarterFrame2(TimeCode),
    TimeCodeQuarterFrame3(TimeCode),
    TimeCodeQuarterFrame4(TimeCode),
    TimeCodeQuarterFrame5(TimeCode),
    TimeCodeQuarterFrame6(TimeCode),
    TimeCodeQuarterFrame7(TimeCode),
    TimeCodeQuarterFrame8(TimeCode),
    /// Max 16383
    SongPosition(u16),
    /// Max 127
    SongSelect(u8),
    TuneRequest,
}

impl SystemCommonMsg {
    pub fn to_midi(&self) -> Vec<u8> {
        let mut r: Vec<u8> = vec![];
        self.extend_midi(&mut r);
        r
    }

    pub fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            SystemCommonMsg::TimeCodeQuarterFrame1(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[0]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame2(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[1]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame3(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[2]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame4(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[3]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame5(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[4]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame6(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[5]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame7(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[6]);
            }
            SystemCommonMsg::TimeCodeQuarterFrame8(qf) => {
                v.push(0xF1);
                v.push(qf.to_nibbles()[7]);
            }
            SystemCommonMsg::SongPosition(pos) => {
                v.push(0xF2);
                let [msb, lsb] = to_u14(*pos);
                v.push(lsb);
                v.push(msb);
            }
            SystemCommonMsg::SongSelect(song) => {
                v.push(0xF3);
                v.push(to_u7(*song));
            }
            SystemCommonMsg::TuneRequest => v.push(0xF6),
        }
    }

    /// Ok results return a MidiMsg and the number of bytes consumed from the input
    pub fn from_midi(_m: &[u8]) -> Result<(Self, usize), &str> {
        Err("TODO: not implemented")
    }
}

impl From<&SystemCommonMsg> for Vec<u8> {
    fn from(m: &SystemCommonMsg) -> Vec<u8> {
        m.to_midi()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn serialize_system_common_msg() {
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TuneRequest
            }
            .to_midi(),
            vec![0xF6]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongSelect(69)
            }
            .to_midi(),
            vec![0xF3, 69]
        );

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::SongPosition(1000)
            }
            .to_midi(),
            vec![0xF2, 0x68, 0x07]
        );

        let frame = TimeCode {
            frame: 40,                     // Should be limited to 29: 0b00011101
            seconds: 58,                   // 0b00111010
            minutes: 20,                   // 0b00010100
            hours: 25,                     // Should be limited to 23: 0b00010111
            code_type: TimeCodeType::DF30, //      0b01000000
        };

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame1(frame)
            }
            .to_midi(),
            vec![0xF1, 0b1101]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame2(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame3(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00100000 + 0b1010]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame4(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00110000 + 0b0011]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame5(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01000000 + 0b0100]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame6(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame7(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01100000 + 0b0111]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::TimeCodeQuarterFrame8(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01110000 + 0b0101]
        );
    }
}
