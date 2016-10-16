#[cfg(feature = "serde")]
use serde::de::{self, Deserialize, Deserializer, SeqVisitor, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

use emulator::device::lem1802::screen::*;

#[cfg(feature = "serde")]
impl Serialize for Vram {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(386));
        for word in self.0.iter() {
            try!(serializer.serialize_seq_elt(&mut state, word));
        }
        serializer.serialize_seq_end(state)
    }
}

#[cfg(feature = "serde")]
impl Deserialize for Vram {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer {
        struct VramVisitor;

        impl Visitor for VramVisitor {
            type Value = Vram;

            fn visit_seq<V>(&mut self,
                            mut visitor: V) -> Result<Vram, V::Error>
                where V: SeqVisitor
            {
                let mut vram = Vram([0; 386]);

                for i in 0..386 as usize {
                    vram.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::end_of_stream()); }
                    };
                }

                try!(visitor.end());

                Ok(vram)
            }
        }

        deserializer.deserialize_seq_fixed_size(386, VramVisitor)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Font {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer {
        let mut state =
            try!(serializer.serialize_seq_fixed_size(256));
        for word in self.0.iter() {
            try!(serializer.serialize_seq_elt(&mut state, word));
        }
        serializer.serialize_seq_end(state)
    }
}

#[cfg(feature = "serde")]
impl Deserialize for Font {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer {
        struct FontVisitor;

        impl Visitor for FontVisitor {
            type Value = Font;

            fn visit_seq<V>(&mut self,
                            mut visitor: V) -> Result<Font, V::Error>
                where V: SeqVisitor
            {
                let mut font = Font([0; 256]);

                for i in 0..256 as usize {
                    font.0[i] = match try!(visitor.visit()) {
                        Some(val) => val,
                        None => { return Err(de::Error::end_of_stream()); }
                    };
                }

                try!(visitor.end());

                Ok(font)
            }
        }

        deserializer.deserialize_seq_fixed_size(256, FontVisitor)
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
