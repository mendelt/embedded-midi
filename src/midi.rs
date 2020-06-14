use crate::error::MidiError;
use embedded_hal::serial::Write;

#[derive(Debug, PartialEq)]
pub enum MidiEvent {
    NoteOn {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
    NoteOff {
        channel: Channel,
        note: Note,
        velocity: Velocity,
    },
}

impl MidiEvent {
    pub fn note_on(channel: Channel, note: Note, velocity: Velocity) -> Self {
        return MidiEvent::NoteOn {
            channel,
            note,
            velocity,
        };
    }

    pub fn note_off(channel: Channel, note: Note, velocity: Velocity) -> Self {
        return MidiEvent::NoteOff {
            channel,
            note,
            velocity,
        };
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Note(u8);

impl From<u8> for Note {
    fn from(note: u8) -> Self {
        Note(note)
    }
}

impl Into<u8> for Note {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Channel(u8);

impl From<u8> for Channel {
    fn from(channel: u8) -> Self {
        Channel(channel)
    }
}

impl Into<u8> for Channel {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Velocity(u8);

impl From<u8> for Velocity {
    fn from(velocity: u8) -> Self {
        Velocity(velocity)
    }
}

impl Into<u8> for Velocity {
    fn into(self) -> u8 {
        self.0
    }
}

pub struct MidiParser {
    state: MidiParserState,
}

enum MidiParserState {
    Empty,
    NoteOnRecvd { channel: u8 },
    NoteOnNoteRecvd { channel: u8, note: u8 },

    NoteOffRecvd { channel: u8 },
    NoteOffNoteRecvd { channel: u8, note: u8 },
}

impl MidiParser {
    /// Initialize midiparser state
    pub fn new() -> Self {
        MidiParser {
            state: MidiParserState::Empty,
        }
    }

    /// Parse midi event byte by byte. Call this whenever a byte is received. When a midi-event is
    /// completed it is returned, otherwise this method updates the internal midiparser state and
    /// and returns none.
    pub fn parse_byte(&mut self, byte: u8) -> Option<MidiEvent> {
        match self.state {
            MidiParserState::Empty => {
                // expect the start of a new message
                let message = byte & 0xf0u8;
                let channel = byte & 0x0fu8;

                match message {
                    0x90 => {
                        self.state = MidiParserState::NoteOnRecvd { channel };
                        None
                    }
                    0x80 => {
                        self.state = MidiParserState::NoteOffRecvd { channel };
                        None
                    }
                    _ => None,
                }
            }
            MidiParserState::NoteOnRecvd { channel } => {
                self.state = MidiParserState::NoteOnNoteRecvd {
                    channel,
                    note: byte,
                };
                None
            }
            MidiParserState::NoteOnNoteRecvd { channel, note } => {
                Some(MidiEvent::note_on(channel.into(), note.into(), byte.into()))
            }
            MidiParserState::NoteOffRecvd { channel } => {
                self.state = MidiParserState::NoteOffNoteRecvd {
                    channel,
                    note: byte,
                };
                None
            }
            MidiParserState::NoteOffNoteRecvd { channel, note } => {
                self.state = MidiParserState::Empty;
                Some(MidiEvent::note_off(
                    channel.into(),
                    note.into(),
                    byte.into(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_note_on() {
        let note_on = MidiEvent::note_on(1.into(), 45.into(), 15.into());

        if let MidiEvent::NoteOn {
            channel,
            note,
            velocity,
        } = note_on
        {
            assert_eq!(channel, Channel(1));
            assert_eq!(note, Note(45));
            assert_eq!(velocity, Velocity(15));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn should_encode_note_off() {
        let note_off = MidiEvent::note_off(0.into(), 0x30.into(), 15.into());

        if let MidiEvent::NoteOff {
            channel,
            note,
            velocity,
        } = note_off
        {
            assert_eq!(channel, Channel(0));
            assert_eq!(note, Note(0x30));
            assert_eq!(velocity, Velocity(15));
        } else {
            assert!(false);
        }
    }
}
