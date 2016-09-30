use std::ops::*;

use types::Register;

pub struct Registers([u16; 8]);

impl Default for Registers {
    fn default() -> Registers {
        Registers([0xdead; 8])
    }
}

impl Index<Register> for Registers {
    type Output = u16;

    fn index(&self, r: Register) -> &u16 {
        &self.0[r as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, r: Register) -> &mut u16 {
        &mut self.0[r as usize]
    }
}
