extern crate midir;
extern crate midi_msg;

use std::io::{stdin, stdout, Write};
use std::error::Error;
use midir::{MidiInput, Ignore};
use midi_msg::*;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}


/// A buffer for TimeCodeQuarterFrameX from which a full TimeCode can
/// be constructed when all 8 Quartes have been received
struct TimeCodeQuarterFrameBuffer {
    buffer: [Option<TimeCode>; 8] 
}

impl TimeCodeQuarterFrameBuffer {
    /// Return a new empty TimeCodeQuarterFrameBuffer
    fn new() -> Self {
        Self {
            buffer: [None; 8]
        }
    }

    /// Add a TimeCodeQuarterFrameX to the buffer, replacing the old one
    fn add(&mut self, message: MidiMsg) {
        match message {
            MidiMsg::SystemCommon { msg } => {
                // Get the target index and timecode from the TimeCodeQuarterFrameX
                let (index, tc) = match msg {
                    SystemCommonMsg::TimeCodeQuarterFrame1(tc) => (0, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame2(tc) => (1, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame3(tc) => (2, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame4(tc) => (3, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame5(tc) => (4, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame6(tc) => (5, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame7(tc) => (6, tc),
                    SystemCommonMsg::TimeCodeQuarterFrame8(tc) => (7, tc),
                    _ => return ()
                };
                // Store the fitting tc at the matching position if is None
                if self.buffer[index].is_none() {
                    self.buffer[index] = Some(tc);
                }
            },
            _ => ()
        }
    }

    /// Return true if all values in the buffer have a value, false otherwise
    fn is_filled(&self) -> bool {
        self.buffer.iter().all(|b| b.is_some())
    }

    /// Construct a Timecode from the TimeCodeQuarterFrames if possible
    fn construct_timecode(&mut self) -> Option<TimeCode> {
        // If the Buffer is not ready, return None
        if !self.is_filled() {
            return None
        }
        // Combine the 4 bit nibbles of the pairs of TimeCode
        // E.g. the low nibble of the frames: u8 stored in TimeCodeQuarter1
        // and the high nibble of frames u8 stored in TimeCodeQuarter2
        let frames: u8 = self.buffer[0].unwrap().frames ^ self.buffer[1].unwrap().frames;
        let seconds: u8 = self.buffer[2].unwrap().seconds ^ self.buffer[3].unwrap().seconds;
        let minutes: u8 = self.buffer[4].unwrap().minutes ^ self.buffer[5].unwrap().minutes;
        let hours: u8 = self.buffer[6].unwrap().hours ^ self.buffer[7].unwrap().hours;
        // The last high TimeCodeQuarter contains the propper code_type so extract it from there
        let code_type = self.buffer[7].unwrap().code_type;
        // Empty the buffer
        self.buffer = [None; 8];
        // Construct and return the TimeCode
        Some(
            TimeCode {
                frames,
                seconds,
                minutes,
                hours,
                code_type,
            }
        )
    }
}


fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();
    
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid input port selected")?
        }
    };
    
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // Create a new buffer for the received TimeCodeQuarterFrames
    let mut quarter_frame_buffer = TimeCodeQuarterFrameBuffer::new();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_stamp, message, _| {
        let (parsed_message, _len) = MidiMsg::from_midi(&message).expect("Not an error");
        
        // Add the message to the TimeCodeQuarterFrameBuffer (ignores every message type
        // other than TimeCodeQuarterFrameX)
        quarter_frame_buffer.add(parsed_message);

        // Construct a timecode if possible
        let maybe_timecode = quarter_frame_buffer.construct_timecode();

        // When we got a timecode, print it out
        if let Some(tc) = maybe_timecode {
            println!("{:0<2}:{:0<2}:{:0<2}.{} ({:?})", tc.hours, tc.minutes, tc.seconds, tc.frames, tc.code_type)
        }
    }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
