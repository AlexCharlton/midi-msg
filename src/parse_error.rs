use alloc::string::String;
use alloc::{fmt};

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
    /// A byte exceeded 7 bits.
    ByteOverflow,
}

// impl error::Error for ParseError {} TODO no_std

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
            }
            Self::Invalid(s) => write!(f, "{}", s),
            Self::ByteOverflow => write!(f, "A byte exceeded 7 bits"),
        }
    }
}
