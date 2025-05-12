use crate::message::Channel;
use crate::parse_error::*;
use crate::util::*;
use alloc::vec::Vec;

/// Intended to act like Control Change messages, but targeted at an individual key.
/// For e.g. Drum sounds that have configurable attack/release/decay per key.
/// Used by [`UniversalRealTimeMsg::KeyBasedInstrumentControl`](crate::UniversalRealTimeMsg::KeyBasedInstrumentControl).
///
/// Defined in CA-023.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBasedInstrumentControl {
    pub channel: Channel,
    /// The MIDI key number.
    pub key: u8,
    /// Any number of (control number, value) pairs.
    ///
    /// Any controller number may be used except Bank Select MSB/LSB (`0x00`, `0x20`),
    /// Data Entry MSB/LSB (`0x06`, `0x26`), RPN/NRPN messages (`0x60` – `0x65`),
    /// and Mode Change messages(`0x78`-`0x7F`).
    ///
    /// Disallowed values will be set to `0x01` (targeting the mod wheel, which probably has no meaning).
    pub control_values: Vec<(u8, u8)>,
}

impl KeyBasedInstrumentControl {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.channel as u8);
        push_u7(self.key, v);
        for (cc, x) in self.control_values.iter().cloned() {
            if cc == 0x06 || cc == 0x26 || cc == 0x60 || cc == 0x65 || cc >= 0x78 {
                v.push(1);
            } else {
                v.push(cc);
            }
            push_u7(x, v);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::NotImplemented("KeyBasedInstrumentControl"))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

    #[test]
    fn serialize_controller_destination() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::KeyBasedInstrumentControl(
                        KeyBasedInstrumentControl {
                            channel: Channel::Ch2,
                            key: 0x60,
                            control_values: vec![
                                (0x06, 0x40), // CC not allowed, should become 0x01
                                (ControlNumber::Effects4Depth as u8, 0x20),
                            ]
                        }
                    ),
                }
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7F, // Receiver device
                0xA, 0x1, // Sysex IDs
                0x1, 0x60, 0x01, 0x40, 94, 0x20, 0xF7
            ]
        );
    }
}
