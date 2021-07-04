// This is a modified version of the MIT-licensed `midir` crate's
// (https://github.com/Boddlnagg/midir) `test_read_inputs.rs` example to use
// MidiMsg::from_midi_with_context() to render the payloads of the resulting
// `MidiMsg` instances instead of displaying the raw bytes.  Note, however, that
// TimingClock messages are suppressed because they are very spammy.
//
// The original source of this file can be found at
// https://github.com/Boddlnagg/midir/blob/master/examples/test_read_input.rs
// specifically, the following revision:
// https://github.com/Boddlnagg/midir/blob/e1064da9b57b480a6ad49fb0c27becb0540cb2d7/examples/test_read_input.rs

extern crate midi_msg;
extern crate midir;

use std::io::{stdin, stdout, Write};
use std::error::Error;

use midi_msg::*;
use midir::{MidiInput, Ignore};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
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

    let mut ctx = ReceiverContext::new();
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, midi_bytes, _| {
        let (msg, _len) = MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx).expect("Not an error");
        
        // Print everything but spammy clock messages.
        if let MidiMsg::SystemRealTime{ msg: SystemRealTimeMsg::TimingClock } = msg {
            // no-op
        } else {
            println!("{}: {:?}", stamp, msg);
        }
    }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}
