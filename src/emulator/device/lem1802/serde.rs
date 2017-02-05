use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, SeqVisitor, Visitor};
use serde::ser::{SerializeSeq};

use emulator::device::lem1802::screen::*;

impl Serialize for Vram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(386));
        for word in self.0.iter() {
            try!(state.serialize_element(word));
        }
        state.end()
    }
}

impl Deserialize for Vram {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer {
        struct VramVisitor;

        impl Visitor for VramVisitor {
            type Value = Vram;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a DCPU Vram")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Vram, V::Error>
                where V: SeqVisitor
            {
                let mut vram = Vram([0; 386]);

                for i in 0..386 as usize {
                    vram.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::invalid_length(386, &self)); }
                    };
                }

                Ok(vram)
            }
        }

        deserializer.deserialize_seq_fixed_size(386, VramVisitor)
    }
}

impl Serialize for Font {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(256));
        for word in self.0.iter() {
            try!(state.serialize_element(word));
        }
        state.end()
    }
}

impl Deserialize for Font {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer {
        struct FontVisitor;

        impl Visitor for FontVisitor {
            type Value = Font;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a DCPU font")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Font, V::Error>
                where V: SeqVisitor
            {
                let mut font = Font([0; 256]);

                for i in 0..256 as usize {
                    font.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::invalid_length(256, &self)); }
                    };
                }

                Ok(font)
            }
        }

        deserializer.deserialize_seq_fixed_size(256, FontVisitor)
    }
}

impl Serialize for Screen {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(SCREEN_SIZE as usize));
        for pixel in self.0.iter() {
            try!(state.serialize_element(pixel));
        }
        state.end()
    }
}

impl Deserialize for Screen {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct ScreenVisitor;

        impl Visitor for ScreenVisitor {
            type Value = Screen;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a DCPU screen")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Screen, V::Error>
                where V: SeqVisitor
            {
                let mut screen = Screen([Color::default();
                                         SCREEN_SIZE as usize]);

                for i in 0..SCREEN_SIZE as usize {
                    screen.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::invalid_length(SCREEN_SIZE as usize, &self)); }
                    };
                }

                Ok(screen)
            }
        }

        deserializer.deserialize_seq_fixed_size(SCREEN_SIZE as usize,
                                                ScreenVisitor)
    }
}
