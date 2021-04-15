use micromath::F32Ext;
use alloc::vec;
use alloc::vec::Vec;
use alloc::format;
use crate::parse_error::*;
use crate::util::*;

/// Global Parameter Control, to control parameters on a device that affect all sound.
/// E.g. a global reverb.
/// Used by [`UniversalRealTimeMsg::GlobalParameterControl`](crate::UniversalRealTimeMsg::GlobalParameterControl).
///
/// As defined in CA-024.
///
/// This C/A is much more permissive than most, and thus has a pretty awkward interface.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalParameterControl {
    /// Between 0 and 127 `SlotPath`s, with each successive path representing a child
    /// of the preceding value. No paths refers to the "top level"
    /// (except if the first value refers to the top level ¯\_(ツ)_/¯)
    pub slot_paths: Vec<SlotPath>,
    /// The number of bytes present in the `id`s of `params`, must be greater than 0
    /// Must line up with the values provided in `params` or output will be massaged
    pub param_id_width: u8,
    /// The number of bytes present in the `value`s of `params, must be greater than 0
    /// Must line up with the values provided in `params` or output will be massaged
    pub value_width: u8,
    /// _Any number_ of `GlobalParameter`s
    pub params: Vec<GlobalParameter>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The type of reverb, used by [`GlobalParameterControl::reverb`].
pub enum ReverbType {
    SmallRoom = 0,
    MediumRoom = 1,
    LargeRoom = 2,
    MediumHall = 3,
    LargeHall = 4,
    Plate = 8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// The type of chorus, used by [`GlobalParameterControl::chorus`].
pub enum ChorusType {
    Chorus1 = 0,
    Chorus2 = 1,
    Chorus3 = 2,
    Chorus4 = 3,
    FBChorus = 4,
    Flanger = 5,
}

impl GlobalParameterControl {
    /// Constructor for a `GlobalParameterControl` directed at a GM2 Reverb slot type.
    ///
    /// `reverb_time` is the time in seconds (0.36 - 9.0) for which the low frequency
    /// portion of the original sound declines by 60dB
    pub fn reverb(reverb_type: Option<ReverbType>, reverb_time: Option<f32>) -> Self {
        let mut params = vec![];

        if let Some(reverb_type) = reverb_type {
            params.push(GlobalParameter {
                id: vec![0],
                value: vec![reverb_type as u8],
            });
        }
        if let Some(reverb_time) = reverb_time {
            params.push(GlobalParameter {
                id: vec![1],
                value: vec![to_u7((reverb_time.ln() / 0.025 + 40.0) as u8)],
            });
        }
        Self {
            slot_paths: vec![SlotPath::Reverb],
            param_id_width: 1,
            value_width: 1,
            params,
        }
    }

    /// Constructor for a `GlobalParameterControl` directed at a GM2 Chorus slot type.
    ///
    /// `mod_rate` is the modulation frequency in Hz (0.0-15.5).
    ///
    /// `mod_depth` is the peak-to-peak swing of the modulation in ms (0.3-40.0).
    ///
    /// `feedback` is the amount of feedback from Chorus output in percent (0.0-97.0).
    ///
    /// `send_to_reverb` is the send level from Chorus to Reverb in percent (0.0-100.0).
    pub fn chorus(
        chorus_type: Option<ChorusType>,
        mod_rate: Option<f32>,
        mod_depth: Option<f32>,
        feedback: Option<f32>,
        send_to_reverb: Option<f32>,
    ) -> Self {
        let mut params = vec![];

        if let Some(chorus_type) = chorus_type {
            params.push(GlobalParameter {
                id: vec![0],
                value: vec![chorus_type as u8],
            });
        }

        if let Some(mod_rate) = mod_rate {
            params.push(GlobalParameter {
                id: vec![1],
                value: vec![to_u7((mod_rate / 0.122) as u8)],
            });
        }

        if let Some(mod_depth) = mod_depth {
            params.push(GlobalParameter {
                id: vec![2],
                value: vec![to_u7(((mod_depth * 3.2) - 1.0) as u8)],
            });
        }

        if let Some(feedback) = feedback {
            params.push(GlobalParameter {
                id: vec![3],
                value: vec![to_u7((feedback / 0.763) as u8)],
            });
        }

        if let Some(send_to_reverb) = send_to_reverb {
            params.push(GlobalParameter {
                id: vec![4],
                value: vec![to_u7((send_to_reverb / 0.787) as u8)],
            });
        }

        Self {
            slot_paths: vec![SlotPath::Chorus],
            param_id_width: 1,
            value_width: 1,
            params,
        }
    }

    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        v.push(self.slot_paths.len().min(127) as u8);
        push_u7(self.param_id_width, v);
        push_u7(self.value_width, v);
        for (i, sp) in self.slot_paths.iter().enumerate() {
            if i > 127 {
                break;
            }
            sp.extend_midi(v);
        }
        for p in self.params.iter() {
            p.extend_midi_with_limits(v, self.param_id_width.max(1), self.value_width.max(1));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// The "slot" of the device being referred to by [`GlobalParameterControl`].
/// Values other than `Unregistered` come from the General MIDI 2 spec.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotPath {
    Reverb,
    Chorus,
    /// For use in paths not described by the GM2 spec
    Unregistered(u8, u8),
}

impl SlotPath {
    pub(crate) fn extend_midi(&self, v: &mut Vec<u8>) {
        match self {
            Self::Reverb => {
                v.push(1);
                v.push(1);
            }
            Self::Chorus => {
                v.push(1);
                v.push(2);
            }
            Self::Unregistered(a, b) => {
                push_u7(*a, v); // MSB first ¯\_(ツ)_/¯
                push_u7(*b, v);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

/// An `id`:`value` pair that must line up with the [`GlobalParameterControl`] that it is placed in.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalParameter {
    pub id: Vec<u8>,
    pub value: Vec<u8>,
}

impl GlobalParameter {
    pub(crate) fn extend_midi_with_limits(
        &self,
        v: &mut Vec<u8>,
        param_id_width: u8,
        value_width: u8,
    ) {
        for i in 0..param_id_width {
            // MSB first
            if let Some(x) = self.id.get(i as usize) {
                push_u7(*x, v);
            } else {
                v.push(0);
            }
        }
        for i in (0..value_width).rev() {
            // LSB first
            if let Some(x) = self.value.get(i as usize) {
                push_u7(*x, v);
            } else {
                v.push(0);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn from_midi(_m: &[u8]) -> Result<(Self, usize), ParseError> {
        Err(ParseError::Invalid(format!("TODO: Not implemented")))
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use alloc::vec;

    #[test]
    fn serialize_global_parameter() {
        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::GlobalParameterControl(GlobalParameterControl {
                        slot_paths: vec![
                            SlotPath::Unregistered(1, 0x47),
                            SlotPath::Unregistered(2, 3)
                        ],
                        param_id_width: 1,
                        value_width: 2,
                        params: vec![
                            GlobalParameter {
                                id: vec![4],
                                value: vec![5, 6, 7] // One byte will be ignored
                            },
                            GlobalParameter {
                                id: vec![4],
                                value: vec![1] // Only the MSB of two bytes
                            }
                        ]
                    }),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7F, // Receiver device
                04, 05, 2,    // Slot path length
                1,    // Param ID width
                2,    // Value width
                1,    // Slot path 1 MSB
                0x47, // Slot path 1 LSB
                2,    // Slot path 2 MSB
                3,    // Slot path 2 LSB
                4,    // Param number 1
                6,    // Param value 1 LSB
                5,    // Param value 1 MSB
                4,    // Param number 2
                0,    // Param value 2 LSB
                1,    // Param value 2 MSB
                0xF7
            ]
        );

        assert_eq!(
            MidiMsg::SystemExclusive {
                msg: SystemExclusiveMsg::UniversalRealTime {
                    device: DeviceID::AllCall,
                    msg: UniversalRealTimeMsg::GlobalParameterControl(
                        GlobalParameterControl::chorus(
                            Some(ChorusType::Flanger),
                            Some(1.1),
                            None,
                            None,
                            Some(100.0)
                        )
                    ),
                },
            }
            .to_midi(),
            vec![
                0xF0, 0x7F, 0x7F, // Receiver device
                04, 05, 1,   // Slot path length
                1,   // Param ID width
                1,   // Value width
                1,   // Slot path 1 MSB
                2,   // Slot path 1 LSB
                0,   // Param number 1: chorus type
                5,   // Param value 1
                1,   // Param number 2: mod rate
                9,   // Param value 2
                4,   // Param number 3: send to reverb
                127, // Param value 3
                0xF7
            ]
        );
    }
}
