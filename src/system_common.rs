use super::time_code::*;
use super::util::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemCommonMsg {
    MTCQuarterFrame1(Frame),
    MTCQuarterFrame2(Frame),
    MTCQuarterFrame3(Frame),
    MTCQuarterFrame4(Frame),
    MTCQuarterFrame5(Frame),
    MTCQuarterFrame6(Frame),
    MTCQuarterFrame7(Frame),
    MTCQuarterFrame8(Frame),
    /// Max 16383
    SongPosition(u16),
    /// Max 127
    SongSelect(u8),
    TuneRequest,
}

impl SystemCommonMsg {
    pub fn to_midi(self) -> Vec<u8> {
        self.into()
    }
}

impl From<SystemCommonMsg> for Vec<u8> {
    fn from(m: SystemCommonMsg) -> Vec<u8> {
        match m {
            SystemCommonMsg::MTCQuarterFrame1(qf) => vec![0xF1, qf.to_nibbles()[0]],
            SystemCommonMsg::MTCQuarterFrame2(qf) => vec![0xF1, qf.to_nibbles()[1]],
            SystemCommonMsg::MTCQuarterFrame3(qf) => vec![0xF1, qf.to_nibbles()[2]],
            SystemCommonMsg::MTCQuarterFrame4(qf) => vec![0xF1, qf.to_nibbles()[3]],
            SystemCommonMsg::MTCQuarterFrame5(qf) => vec![0xF1, qf.to_nibbles()[4]],
            SystemCommonMsg::MTCQuarterFrame6(qf) => vec![0xF1, qf.to_nibbles()[5]],
            SystemCommonMsg::MTCQuarterFrame7(qf) => vec![0xF1, qf.to_nibbles()[6]],
            SystemCommonMsg::MTCQuarterFrame8(qf) => vec![0xF1, qf.to_nibbles()[7]],
            SystemCommonMsg::SongPosition(pos) => {
                let [msb, lsb] = to_u14(pos);
                vec![0xF2, lsb, msb]
            }
            SystemCommonMsg::SongSelect(song) => vec![0xF3, to_u7(song)],
            SystemCommonMsg::TuneRequest => vec![0xF6],
        }
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

        let frame = Frame {
            frame: 40,                     // Should be limited to 29: 0b00011101
            seconds: 58,                   // 0b00111010
            minutes: 20,                   // 0b00010100
            hours: 25,                     // Should be limited to 23: 0b00010111
            code_type: TimeCodeType::DF30, //      0b01000000
        };

        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame1(frame)
            }
            .to_midi(),
            vec![0xF1, 0b1101]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame2(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame3(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00100000 + 0b1010]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame4(frame)
            }
            .to_midi(),
            vec![0xF1, 0b00110000 + 0b0011]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame5(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01000000 + 0b0100]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame6(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01010000 + 0b0001]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame7(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01100000 + 0b0111]
        );
        assert_eq!(
            MidiMsg::SystemCommon {
                msg: SystemCommonMsg::MTCQuarterFrame8(frame)
            }
            .to_midi(),
            vec![0xF1, 0b01110000 + 0b0101]
        );
    }
}
