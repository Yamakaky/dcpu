use std::fmt;

#[cfg(feature = "serde")]
use serde::de::{self, Deserialize, Deserializer, SeqVisitor, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

use emulator::device::lem1802::*;

pub struct Screen(pub [Color; SCREEN_SIZE as usize]);

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A beautiful screen with shiny pixels")
    }
}

#[cfg(feature = "serde")]
impl Serialize for Screen {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(SCREEN_SIZE as usize));
        for pixel in self.0.iter() {
            try!(serializer.serialize_seq_elt(&mut state, pixel));
        }
        serializer.serialize_seq_end(state)
    }
}

#[cfg(feature = "serde")]
impl Deserialize for Screen {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer {
        struct ScreenVisitor;

        impl Visitor for ScreenVisitor {
            type Value = Screen;

            fn visit_seq<V>(&mut self,
                            mut visitor: V) -> Result<Screen, V::Error>
                where V: SeqVisitor
            {
                let mut screen = Screen([Color::default();
                                         SCREEN_SIZE as usize]);

                for i in 0..SCREEN_SIZE as usize {
                    screen.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::end_of_stream()); }
                    };
                }

                try!(visitor.end());

                Ok(screen)
            }
        }

        deserializer.deserialize_seq_fixed_size(SCREEN_SIZE as usize,
                                                ScreenVisitor)
    }
}

#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub blinking: bool,
}

impl Color {
    pub fn from_packed(c: u16) -> Color {
        Color {
            r: ((c >> 8) & 0xf) as f32 / 0xf as f32,
            g: ((c >> 4) & 0xf) as f32 / 0xf as f32,
            b: ( c        & 0xf) as f32 / 0xf as f32,
            blinking: false,
        }
    }
}
