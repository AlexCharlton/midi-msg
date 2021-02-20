# midi-msg

The goal of midi-msg is to be a complete representation of the MIDI 1.0 Detailed Specification and its many extensions and addenda, to allow for the serialization and deserialization of byte streams into a typed representation. MIDI 2.0 may be supported at a later date.

midi-msg types adhere to the taxonomy detailed in the MIDI spec, and have the goal of being entirely safe. That is to say, any `MidiMsg` can be serialized into a valid MIDI byte sequence. Additionally, midi-msg strives to capture the semantic meaning of MIDI with types that are not simply "bags of bytes". Any values that are not simple numeric ones are represented with their meaning in mind. Nonetheless, much of what MIDI achieves is predicated on passing around numeric values and as such they are handled according to the following approach.

Since the MIDI spec makes extensive use of non-byte-aligned integers, a Rust representation could either be achieved by introducing "exotic" integer types or by clamping Rust's primitive types to the desired range. For the sake of ergonomics, the latter approach was taken, though this does make this typed representation slightly "lossy". Any overflows between these representations are treated as max values (or min, for negative signed values). Other libraries, which have taken other approaches with respect to these design decisions, can be found below.


## To be implemented
- [ ] Deserialization


## Other Rust MIDI representation libraries
- [apres](https://crates.io/crates/apres)
- [helgoboss-midi](https://crates.io/crates/helgoboss-midi)
- [midi-control](https://crates.io/crates/midi-control)
- [midi-types](https://crates.io/crates/midi-types)
- [midly](https://crates.io/crates/midly)
