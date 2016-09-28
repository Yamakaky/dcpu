use std;
use std::ops::*;
use std::num::Wrapping;

pub type InnerRam = [u16; 0x10000];
pub struct Ram(pub InnerRam);

impl Ram {
    pub fn copy<'a, T: Iterator<Item=&'a u16>>(&mut self, items: T, offset: u16) {
        let (low, high) = self.0.split_at_mut(offset as usize);
        for (from, to) in items.zip(high.iter_mut()
                                        .chain(low.iter_mut())) {
            *to = *from;
        }
    }

    pub fn iter_wrap(&self, offset: u16) -> Iter {
        self.iter()
            .skip(offset as usize)
            .chain(self.iter()
                       .take(offset as usize))
    }
}

// Waiting for `-> impl Iterator`...
pub type Iter<'a> = std::iter::Chain<std::iter::Skip<std::slice::Iter<'a, u16>>,
                                 std::iter::Take<std::slice::Iter<'a, u16>>>;

impl Deref for Ram {
    type Target = InnerRam;

    fn deref(&self) -> &InnerRam {
        &self.0
    }
}

impl DerefMut for Ram {
    fn deref_mut(&mut self) -> &mut InnerRam {
        &mut self.0
    }
}

impl Index<u16> for Ram {
    type Output = u16;

    fn index(&self, i: u16) -> &u16{
        &self.0[i as usize]
    }
}

impl Index<Wrapping<u16>> for Ram {
    type Output = u16;

    fn index(&self, i: Wrapping<u16>) -> &u16{
        &self.0[i.0 as usize]
    }
}

impl Index<Range<u16>> for Ram {
    type Output = [u16];

    fn index(&self, i: Range<u16>) -> &[u16] {
        &self.0[i.start as usize..i.end as usize]
    }
}

impl Index<RangeFrom<u16>> for Ram {
    type Output = [u16];

    fn index(&self, i: RangeFrom<u16>) -> &[u16] {
        &self.0[i.start as usize..]
    }
}

impl Index<RangeTo<u16>> for Ram {
    type Output = [u16];

    fn index(&self, i: RangeTo<u16>) -> &[u16] {
        &self.0[..i.end as usize]
    }
}

impl IndexMut<u16> for Ram {
    fn index_mut(&mut self, i: u16) -> &mut u16{
        &mut self.0[i as usize]
    }
}

impl IndexMut<Wrapping<u16>> for Ram {
    fn index_mut(&mut self, i: Wrapping<u16>) -> &mut u16{
        &mut self.0[i.0 as usize]
    }
}

impl IndexMut<Range<u16>> for Ram {
    fn index_mut(&mut self, i: Range<u16>) -> &mut [u16] {
        &mut self.0[i.start as usize..i.end as usize]
    }
}

impl IndexMut<RangeFrom<u16>> for Ram {
    fn index_mut(&mut self, i: RangeFrom<u16>) -> &mut [u16] {
        &mut self.0[i.start as usize..]
    }
}

impl IndexMut<RangeTo<u16>> for Ram {
    fn index_mut(&mut self, i: RangeTo<u16>) -> &mut [u16] {
        &mut self.0[..i.end as usize]
    }
}
