# midi-msg

midi-msg aims to be a complete representation of the MIDI 1.0 Detailed Specification and its many extensions and addenda, to allow for the serialization and deserialization of byte streams to and from a typed representation. MIDI 2.0 may be supported at a later date.

midi-msg types follow the taxonomy detailed in the MIDI spec, and have the goal of being entirely safe. That is to say, any `MidiMsg` can be serialized into a valid MIDI byte sequence. Likewise, as it strives for completeness, any valid MIDI byte sequence can be deserialized into a `MidiMsg`. Additionally, midi-msg strives to capture the semantic meaning of MIDI with types that are not simply "bags of bytes". Any values that are not numeric atoms are represented with their meaning in mind. Nonetheless, much of what MIDI achieves is predicated on passing around numeric values and as such they are handled according to the following approach.

Since the MIDI spec makes extensive use of non-byte-aligned integers, a Rust representation could either be achieved by introducing "exotic" integer types or by clamping Rust's primitive types to the desired range. For the sake of ergonomics, the latter approach was taken, though this does make this typed representation slightly "lossy". Any overflows between these representations are treated as max values (or min, for negative signed values). Other libraries, which have taken other approaches with respect to these design decisions, can be found below.


## To be implemented
- [ ] Deserialization


## Support 
The following [MMA documents](MIDI 1.0 Detailed Specification) are supported (with their corresponded Midi Manufacturer Association [MMA] publication number, Recommended Practice [RP] number, or Changes/Additions [CA] number as noted):
- MIDI 1.0 Detailed Specification 4.2.1 (The base specification. Reference of types should be assumed to be this document unless otherwise specified)
- MIDI Time Code (MMA0001 / RP004 / RP008)
- General MIDI System Level 1 (MMA0007 / RP003)
- General MIDI 2 1.2 (RP-024/RP-036/RP-037/RP-045)
- MIDI Tuning Updated Specification (CA-020/CA-021/RP-020)
- Controller Destination Setting (CA-022)
- Global Parameter Control (CA-024)
- Master Fine/Coarse Tuning (CA-025)
- Modulation Depth Range RPN (CA-026)
- CC #88 High Resolution Velocity Prefix (CA-031)
- Response to Data Inc/Dec Controllers (RP-018)
- Sound Controller Defaults (RP-021)
- Redefinition of RPN 01/02 (RP-022)
- Renaming of CC91 and CC93 (RP-023)
- MIDI Polyphonic Expression 1.0 (RP-053)


The following addenda are not yet fully supported by midi-msg, though hooks are provided to access these messages:

- MIDI Machine Control 1.0 (MMA0016 / RP013) (partial support)
- MIDI Show Control 1.1.1 (RP002/RP014)


The following addenda were consulted but considered not relevant to this library:

- Response to Reset All Controllers (RP-018)
- Default Pan Formula (RP-036)


Support for the Standard MIDI Files specification may be added.


## Other Rust MIDI representation libraries
- [apres](https://crates.io/crates/apres)
- [helgoboss-midi](https://crates.io/crates/helgoboss-midi)
- [midi-control](https://crates.io/crates/midi-control)
- [midi-types](https://crates.io/crates/midi-types)
- [midly](https://crates.io/crates/midly)


## Contributing
Pull requests for the features listed above as not-yet supported, for bug fixes (any omissions from the spec are considered bugs), or for documentation additions are most welcome.
