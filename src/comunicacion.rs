pub use self::Message::{Error, Pay, Succesfull, Unknown};

pub enum Message {
    Error,
    Pay,
    Succesfull,
    Unknown,
}

impl From<u8> for Message {
    fn from(code: u8) -> Message {
        match code & 0xF0 {
            0x00 => Pay,
            0x10 => Succesfull,
            0x20 => Error,
            _ => Unknown,
        }
    }
}

impl From<Message> for u8 {
    fn from(code: Message) -> u8 {
        match code {
            Pay => 0x00,
            Succesfull => 0x10,
            Error => 0x20,
            _ => 0x99,
        }
    }
}