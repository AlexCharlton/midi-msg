use alloc::fmt;
#[cfg(feature = "std")]
use std::error;
/// Returned when [`MidiMsg::from_midi`](crate::MidiMsg::from_midi) and similar where not successful.
#[derive(Debug)]
pub enum ParseError {
    /// The given input ended before a `MidiMsg` could be fully formed.
    UnexpectedEnd,
    /// Received a non-status byte with no prior channel messages.
    ContextlessRunningStatus,
    /// Reached end without an End of System Exclusive flag.
    NoEndOfSystemExclusiveFlag,
    /// Encountered an unexpected End of System Exclusive flag.
    UnexpectedEndOfSystemExclusiveFlag,
    /// Received a system exclusive message but the crate
    /// was built without the sysex feature.
    SystemExclusiveDisabled,
    /// The series of bytes was otherwise invalid.
    Invalid(&'static str),
    /// Attempted to use a not yet implemented feature.
    NotImplemented(&'static str),
    /// A byte exceeded 7 bits.
    ByteOverflow,
    /// Encountered an undefined system common message
    UndefinedSystemCommonMessage(u8),
    /// Encountered an undefined system real time message
    UndefinedSystemRealTimeMessage(u8),
    /// Encountered an undefined system exclusive message
    UndefinedSystemExclusiveMessage(Option<u8>)
}

#[cfg(feature = "std")]
impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error parsing MIDI input: ")?;
        match self {
            Self::UnexpectedEnd => {
                write!(f, "The input ended before a MidiMsg could be fully formed")
            }
            Self::ContextlessRunningStatus => write!(
                f,
                "Received a non-status byte with no prior channel messages"
            ),
            Self::NoEndOfSystemExclusiveFlag => {
                write!(f, "Tried to read a SystemExclusiveMsg, but reached the end without an End of System Exclusive flag")
            },
            Self::UnexpectedEndOfSystemExclusiveFlag => {
                write!(f, "Encountered an unexpected End of System Exclusive flag")
            }
            Self::SystemExclusiveDisabled => {
                write!(f, "Received a system exclusive message but the crate was built without the sysex feature")
            }
            Self::NotImplemented(msg) => {
                write!(f, "{} is not yet implemented", msg)
            },
            Self::Invalid(s) => write!(f, "{}", s),
            Self::ByteOverflow => write!(f, "A byte exceeded 7 bits"),
            Self::UndefinedSystemCommonMessage(byte) => write!(f, "Encountered undefined system common message {:#04x}", byte),
            Self::UndefinedSystemRealTimeMessage(byte) => write!(f, "Encountered undefined system real time message {:#04x}", byte),
            Self::UndefinedSystemExclusiveMessage(byte) => {
                if let Some(byte) = byte {
                    write!(f, "Encountered undefined system exclusive message {:#04x}", byte)
                } else {
                    write!(f, "Encountered undefined system exclusive message {:?}", byte)
                }

            },
        }
    }
}
