use alloc::string::String;
use alloc::{fmt};
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
    /// The series of bytes was otherwise invalid.
    Invalid(String),
    /// Attempted to use a not yet implemented feature.
    NotImplemented(&'static str),
    /// A byte exceeded 7 bits.
    ByteOverflow,
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
            Self::NotImplemented(msg) => {
                write!(f, "{} is not yet implemented", msg)
            },
            Self::Invalid(s) => write!(f, "{}", s),
            Self::ByteOverflow => write!(f, "A byte exceeded 7 bits"),
        }
    }
}
