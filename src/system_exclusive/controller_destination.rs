use alloc::vec::Vec;
use alloc::format;
use crate::message::Channel;
use crate::parse_error::*;
use crate::util::*;

/// Allows for the selection of the destination of a channel pressure/poly key pressure message.
/// Used by [`UniversalRealTimeMsg`](crate::UniversalRealTimeMsg).
///
/// Defined in CA-022.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerDestination {
    pub channel: Channel,
    /// Any number of (ControlledParameter, range) pairs
    pub param_ranges: Vec<(ControlledParameter, u8)>,
}

impl ControllerDestination {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.channel as u8);
        for (p, r) in self.param_ranges.iter() {
            v.push(*p as u8);
            push_u7(*r, v);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ControllerDestination not implemented")))
    }
}

/// Allows for the selection of the destination of a control change message.
/// Used by [`UniversalRealTimeMsg::GlobalParameterControl`](crate::UniversalRealTimeMsg::GlobalParameterControl).
///
/// Defined in CA-022.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlChangeControllerDestination {
    pub channel: Channel,
    /// A control number between `0x01` - `0x1F` or `0x40` - `0x5F`
    /// Values outside these ranges will be coerced
    pub control_number: u8,
    /// Any number of (ControlledParameter, range) pairs
    pub param_ranges: Vec<(ControlledParameter, u8)>,
}

impl ControlChangeControllerDestination {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.channel as u8);
        if self.control_number < 0x40 {
            v.push(self.control_number.max(0x01).min(0x1F));
        } else {
            v.push(self.control_number.max(0x40).min(0x5F));
        }
        for (p, r) in self.param_ranges.iter() {
            v.push(*p as u8);
            push_u7(*r, v);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: ControlChangeControllerDestination not implemented")))
    }
}
/// The parameters that can be controlled by [`ControllerDestination`] or
/// [`ControlChangeControllerDestination`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledParameter {
    PitchControl = 0,
    FilterCutoffControl = 1,
    AmplitudeControl = 2,
    LFOPitchDepth = 3,
    LFOFilterDepth = 4,
    LFOAmplitudeDepth = 5,
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
                    msg: UniversalRealTimeMsg::ControlChangeControllerDestination(
                        ControlChangeControllerDestination {
                            channel: Channel::Ch2,
                            control_number: 0x50,
                            param_ranges: vec![
                                (ControlledParameter::PitchControl, 0x42),
                                (ControlledParameter::FilterCutoffControl, 0x60)
                            ]
                        }
                    ),
                }
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7F, // Receiver device
                09, 03, // Sysex IDs
                01, 0x50, 0, 0x42, 1, 0x60, 0xF7
            ]
        );
    }
}
