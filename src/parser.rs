use crate::MidiEvent;

pub struct MidiParser {
    state: MidiParserState,
}

enum MidiParserState {
    Idle,
    NoteOnRecvd { channel: u8 },
    NoteOnNoteRecvd { channel: u8, note: u8 },

    NoteOffRecvd { channel: u8 },
    NoteOffNoteRecvd { channel: u8, note: u8 },

    ControlChangeRecvd { channel: u8 },
    ControlChangeControlRecvd { channel: u8, control: u8 },

    ProgramChangeRecvd { channel: u8 },

    ChannelPressureRecvd { channel: u8 },

    PitchBendRecvd { channel: u8 },
    PitchBendFirstByteRecvd { channel: u8, byte1: u8 },
}

fn is_status_byte(byte: u8) -> bool {
    byte & 0x80 == 0x80
}

fn split_message_and_channel(byte: u8) -> (u8, u8) {
    (byte & 0xf0u8, byte & 0x0fu8)
}

impl MidiParser {
    /// Initialize midiparser state
    pub fn new() -> Self {
        MidiParser {
            state: MidiParserState::Idle,
        }
    }

    /// Parse midi event byte by byte. Call this whenever a byte is received. When a midi-event is
    /// completed it is returned, otherwise this method updates the internal midiparser state and
    /// and returns none.
    pub fn parse_byte(&mut self, byte: u8) -> Option<MidiEvent> {
        if is_status_byte(byte) {
            let (message, channel) = split_message_and_channel(byte);

            match message {
                0x80 => {
                    self.state = MidiParserState::NoteOffRecvd { channel };
                    None
                }
                0x90 => {
                    self.state = MidiParserState::NoteOnRecvd { channel };
                    None
                }
                0xB0 => {
                    self.state = MidiParserState::ControlChangeRecvd { channel };
                    None
                }
                0xC0 => {
                    self.state = MidiParserState::ProgramChangeRecvd { channel };
                    None
                }
                0xD0 => {
                    self.state = MidiParserState::ChannelPressureRecvd { channel };
                    None
                }
                0xE0 => {
                    self.state = MidiParserState::PitchBendRecvd { channel };
                    None
                }
                _ => None,
            }
        } else {
            match self.state {
                MidiParserState::NoteOnRecvd { channel } => {
                    self.state = MidiParserState::NoteOnNoteRecvd {
                        channel,
                        note: byte,
                    };
                    None
                }
                MidiParserState::NoteOnNoteRecvd { channel, note } => {
                    self.state = MidiParserState::NoteOnRecvd { channel };
                    Some(MidiEvent::NoteOn {
                        channel: channel.into(),
                        note: note.into(),
                        velocity: byte.into(),
                    })
                }

                MidiParserState::NoteOffRecvd { channel } => {
                    self.state = MidiParserState::NoteOffNoteRecvd {
                        channel,
                        note: byte,
                    };
                    None
                }
                MidiParserState::NoteOffNoteRecvd { channel, note } => {
                    self.state = MidiParserState::NoteOffRecvd { channel };
                    Some(MidiEvent::NoteOff {
                        channel: channel.into(),
                        note: note.into(),
                        velocity: byte.into(),
                    })
                }

                MidiParserState::ControlChangeRecvd { channel } => {
                    self.state = MidiParserState::ControlChangeControlRecvd {
                        channel,
                        control: byte,
                    };
                    None
                }
                MidiParserState::ControlChangeControlRecvd { channel, control } => {
                    self.state = MidiParserState::ControlChangeRecvd { channel };
                    Some(MidiEvent::ControlChange {
                        channel: channel.into(),
                        control: control.into(),
                        value: byte.into(),
                    })
                }

                MidiParserState::ProgramChangeRecvd { channel } => Some(MidiEvent::ProgramChange {
                    channel: channel.into(),
                    program: byte.into(),
                }),

                MidiParserState::ChannelPressureRecvd { channel } => {
                    Some(MidiEvent::ChannelPressure {
                        channel: channel.into(),
                        value: byte.into(),
                    })
                }

                MidiParserState::PitchBendRecvd { channel } => {
                    self.state = MidiParserState::PitchBendFirstByteRecvd {
                        channel,
                        byte1: byte,
                    };
                    None
                }
                MidiParserState::PitchBendFirstByteRecvd { channel, byte1 } => {
                    self.state = MidiParserState::PitchBendRecvd { channel };
                    Some(MidiEvent::PitchBend {
                        channel: channel.into(),
                        value: (byte1, byte).into(),
                    })
                }
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[test]
    fn should_parse_status_byte() {
        assert!(is_status_byte(0x80u8));
        assert!(is_status_byte(0x94u8));
        assert!(!is_status_byte(0x00u8));
        assert!(!is_status_byte(0x78u8));
    }

    #[test]
    fn should_split_message_and_channel() {
        let (message, channel) = split_message_and_channel(0x91u8);
        assert_eq!(message, 0x90u8);
        assert_eq!(channel, 1);
    }

    #[test]
    fn should_parse_note_on() {
        MidiParser::new().assert_result(
            &[0x91, 0x04, 0x34],
            &[MidiEvent::NoteOn {
                channel: 1.into(),
                note: 4.into(),
                velocity: 0x34.into(),
            }],
        );
    }

    #[test]
    fn should_handle_note_on_running_state() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x76, 0x34, // First note_on
                0x33, 0x65, // Second note on without status byte
            ],
            &[
                MidiEvent::NoteOn {
                    channel: 2.into(),
                    note: 0x76.into(),
                    velocity: 0x34.into(),
                },
                MidiEvent::NoteOn {
                    channel: 2.into(),
                    note: 0x33.into(),
                    velocity: 0x65.into(),
                },
            ],
        );
    }

    #[test]
    fn should_parse_note_off() {
        MidiParser::new().assert_result(
            &[0x82, 0x76, 0x34],
            &[MidiEvent::NoteOff {
                channel: 2.into(),
                note: 0x76.into(),
                velocity: 0x34.into(),
            }],
        );
    }

    #[test]
    fn should_handle_note_off_running_state() {
        MidiParser::new().assert_result(
            &[
                0x82, 0x76, 0x34, // First note_off
                0x33, 0x65, // Second note_off without status byte
            ],
            &[
                MidiEvent::NoteOff {
                    channel: 2.into(),
                    note: 0x76.into(),
                    velocity: 0x34.into(),
                },
                MidiEvent::NoteOff {
                    channel: 2.into(),
                    note: 0x33.into(),
                    velocity: 0x65.into(),
                },
            ],
        );
    }

    #[test]
    fn should_parse_control_change() {
        MidiParser::new().assert_result(
            &[0xB2, 0x76, 0x34],
            &[MidiEvent::ControlChange {
                channel: 2.into(),
                control: 0x76.into(),
                value: 0x34.into(),
            }],
        );
    }

    #[test]
    fn should_parse_control_change_running_state() {
        MidiParser::new().assert_result(
            &[
                0xb3, 0x3C, 0x18, // First control change
                0x43, 0x01, // Second control change without status byte
            ],
            &[
                MidiEvent::ControlChange {
                    channel: 3.into(),
                    control: 0x3C.into(),
                    value: 0x18.into(),
                },
                MidiEvent::ControlChange {
                    channel: 3.into(),
                    control: 0x43.into(),
                    value: 0x01.into(),
                },
            ],
        );
    }

    #[test]
    fn should_parse_program_change() {
        MidiParser::new().assert_result(
            &[0xC9, 0x15],
            &[MidiEvent::ProgramChange {
                channel: 9.into(),
                program: 0x15.into(),
            }],
        );
    }

    #[test]
    fn should_parse_program_change_running_state() {
        MidiParser::new().assert_result(
            &[
                0xC3, 0x67, // First program change
                0x01, // Second program change without status byte
            ],
            &[
                MidiEvent::ProgramChange {
                    channel: 3.into(),
                    program: 0x67.into(),
                },
                MidiEvent::ProgramChange {
                    channel: 3.into(),
                    program: 0x01.into(),
                },
            ],
        );
    }

    #[test]
    fn should_parse_channel_pressure() {
        MidiParser::new().assert_result(
            &[0xDD, 0x37],
            &[MidiEvent::ChannelPressure {
                channel: 13.into(),
                value: 0x37.into(),
            }],
        );
    }

    #[test]
    fn should_parse_channel_pressure_running_state() {
        MidiParser::new().assert_result(
            &[
                0xD6, 0x77, // First channel pressure
                0x43, // Second channel pressure without status byte
            ],
            &[
                MidiEvent::ChannelPressure {
                    channel: 6.into(),
                    value: 0x77.into(),
                },
                MidiEvent::ChannelPressure {
                    channel: 6.into(),
                    value: 0x43.into(),
                },
            ],
        );
    }

    #[test]
    fn should_parse_pitchbend() {
        MidiParser::new().assert_result(
            &[0xE8, 0x14, 0x56],
            &[MidiEvent::PitchBend {
                channel: 8.into(),
                value: (0x14, 0x56).into(),
            }],
        );
    }

    #[test]
    fn should_parse_pitchbend_running_state() {
        MidiParser::new().assert_result(
            &[
                0xE3, 0x3C, 0x18, // First pitchbend
                0x43, 0x01, // Second pitchbend without status byte
            ],
            &[
                MidiEvent::PitchBend {
                    channel: 3.into(),
                    value: (0x3C, 0x18).into(),
                },
                MidiEvent::PitchBend {
                    channel: 3.into(),
                    value: (0x43, 0x01).into(),
                },
            ],
        );
    }

    #[test]
    fn should_ignore_incomplete_messages() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x1b, // Start note off message
                0x82, 0x76, 0x34, // continue with a complete note on message
            ],
            &[MidiEvent::NoteOff {
                channel: 2.into(),
                note: 0x76.into(),
                velocity: 0x34.into(),
            }],
        );
    }

    impl MidiParser {
        /// Test helper function, asserts if a slice of bytes parses to some set of midi events
        fn assert_result(&mut self, bytes: &[u8], expected_events: &[MidiEvent]) {
            let events: Vec<MidiEvent> = bytes
                .into_iter()
                .filter_map(|byte| self.parse_byte(*byte))
                .collect();

            assert_eq!(expected_events, events.as_slice());
        }
    }
}
