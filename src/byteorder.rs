use std::io::prelude::*;
use std::io;
use std::marker::PhantomData;

pub trait WriteBytesExt: Write {
    fn write_item<I: Item, B: ByteOrder>(&mut self, item: I) -> io::Result<()> {
        let mut data = item.encode_to_big_endian();
        B::encode_to_big_endian::<I>(&mut data);
        self.write_all(data.as_ref())
    }
}
pub trait ReadBytesExt: Read {
    fn read_item<I: Item, B: ByteOrder>(&mut self) -> io::Result<I> {
        let mut buf = I::Buf::default();
        try!(self.read_exact(buf.as_mut()));
        B::decode_from_big_endian::<I>(&mut buf);
        Ok(I::decode_from_big_endian(&buf))
    }

    fn iter_items<I: Item, B: ByteOrder>(&mut self) -> Iter<Self, I, B> {
        Iter {
            inner: self,
            _phantom: PhantomData,
        }
    }
}

pub struct Iter<'a, R: ReadBytesExt + ?Sized + 'a, I, B> {
    inner: &'a mut R,
    _phantom: PhantomData<(I, B)>,
}

impl<'a, R: ReadBytesExt + ?Sized + 'a, I: Item, B: ByteOrder> Iterator for Iter<'a, R, I, B> {
    type Item = I;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.read_item::<I, B>().ok()
    }
}

impl<W: Read + ?Sized> ReadBytesExt for W {}
impl<W: Write + ?Sized> WriteBytesExt for W {}

pub trait ByteOrder {
    fn encode_to_big_endian<I: Item>(&mut I::Buf);
    fn decode_from_big_endian<I: Item>(&mut I::Buf);
}

pub enum BigEndian {}
pub enum LittleEndian {}

impl ByteOrder for BigEndian {
    fn encode_to_big_endian<I: Item>(_: &mut I::Buf) {
    }
    fn decode_from_big_endian<I: Item>(_: &mut I::Buf) {
    }
}
impl ByteOrder for LittleEndian {
    fn encode_to_big_endian<I: Item>(buf: &mut I::Buf) {
        buf.as_mut().reverse();
    }
    fn decode_from_big_endian<I: Item>(buf: &mut I::Buf) {
        buf.as_mut().reverse();
    }
}

pub trait Item {
    type Buf: AsRef<[u8]> + AsMut<[u8]> + Default;
    fn encode_to_big_endian(&self) -> Self::Buf;
    fn decode_from_big_endian(&Self::Buf) -> Self;
}

impl Item for u16 {
    type Buf = [u8; 2];

    fn encode_to_big_endian(&self) -> Self::Buf {
        [(*self >> 8) as u8, *self as u8]
    }

    fn decode_from_big_endian(buf: &Self::Buf) -> u16 {
        (buf[0] as u16) << 8 | buf[1] as u16
    }
}
