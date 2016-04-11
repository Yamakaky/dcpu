use std::iter::Iterator;

use types::*;

pub struct U16ToInstruction<I> {
    it: I,
    buffer: [u16; 3],
    len_buffer: usize
}

impl<I: Iterator<Item=u16>> U16ToInstruction<I> {
    pub fn chain(it: I) -> U16ToInstruction<I> {
        U16ToInstruction {
            it: it,
            buffer: [0; 3],
            len_buffer: 0
        }
    }
}

impl<I: Iterator<Item=u16>> Iterator for U16ToInstruction<I> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Instruction> {
        while self.len_buffer < 3 {
            if let Some(u) = self.it.next() {
                self.buffer[self.len_buffer] = u;
                self.len_buffer += 1;
            } else {
                break;
            }
        }

        let (used, i) = match Instruction::decode(&self.buffer) {
            Ok(x) => x,
            Err(_) => return None
        };
        if used as usize > self.len_buffer {
            return None;
        }

        self.len_buffer -= used as usize;
        Some(i)
    }
}

pub struct InstructionToU16<I> {
    it: I,
    buffer: [u16; 3],
    len_buffer: usize
}

impl<I: Iterator<Item=Instruction>> InstructionToU16<I> {
    pub fn chain(it: I) -> InstructionToU16<I> {
        InstructionToU16 {
            it: it,
            buffer: [0; 3],
            len_buffer: 0
        }
    }
}

impl<I: Iterator<Item=Instruction>> Iterator for InstructionToU16<I> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        if self.len_buffer == 0 {
            if let Some(i) = self.it.next() {
                self.len_buffer = i.encode(&mut self.buffer) as usize;
            } else {
                return None;
            }
        }

        self.len_buffer -= 1;
        Some(self.buffer[self.len_buffer])
    }
}
