use std::fmt::Debug;
use std::num::Wrapping;

use num::traits::FromPrimitive;

use cpu::Cpu;
use device::*;

const MASK_INDEX: u16 = 0xf;
pub const SCREEN_HEIGHT: u16 = 128;
pub const SCREEN_WIDTH: u16 = 96;
const CHAR_HEIGHT: u16 = 8;
const CHAR_WIDTH: u16 = 4;
const NB_CHARS: u16 = (SCREEN_HEIGHT / CHAR_HEIGHT) * (SCREEN_WIDTH / CHAR_WIDTH);

const MASK_BLINKING: u16 = 1 << 7;
const MASK_COLOR_IDX: u16 = 0xf;
const MASK_CHAR: u16 = 0x7f;
const SHIFT_FG: u16 = 12;
const SHIFT_BG: u16 = 8;

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    MEM_MAP_SCREEN = 0x0,
    MEM_MAP_FONT = 0x1,
    MEM_MAP_PALETTE = 0x2,
    SET_BORDER_COLOR = 0x3,
}
}

#[derive(Default, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub blinking: bool,
}
pub type Screen = [Color; (SCREEN_HEIGHT * SCREEN_WIDTH) as usize];

impl Color {
    fn from_packed(c: u16) -> Color {
        Color {
            r: ((c >> 8) & 0xf) as f32 / 0xf as f32,
            g: ((c >> 4) & 0xf) as f32 / 0xf as f32,
            b: ((c >> 0) & 0xf) as f32 / 0xf as f32,
            blinking: false,
        }
    }
}

pub trait Backend: Debug {
    fn tick<B: Backend>(&self, &Cpu, &LEM1802<B>, tick_count: u64);
}

#[derive(Debug)]
pub struct LEM1802<B: Backend> {
    video_map: Wrapping<u16>,
    font_map: Wrapping<u16>,
    palette_map: Wrapping<u16>,
    border_color_index: u16,
    backend: B,
}

impl<B: Backend> LEM1802<B> {
    pub fn new(backend: B) -> LEM1802<B> {
        LEM1802 {
            video_map: Wrapping(0),
            font_map: Wrapping(0),
            palette_map: Wrapping(0),
            border_color_index: 0,
            backend: backend,
        }
    }
}

impl<B: Backend> Device for LEM1802<B> {
    fn hardware_id(&self) -> u32 {
        0x7349f615
    }

    fn hardware_version(&self) -> u16 {
        0x1802
    }

    fn manufacturer(&self) -> u32 {
        0x1c6c8b36
    }

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay, ()> {
        let a = cpu.registers[0];
        let b = cpu.registers[1];
        match Command::from_u16(a) {
            Some(Command::MEM_MAP_SCREEN) => self.video_map = Wrapping(b),
            Some(Command::MEM_MAP_FONT) => self.font_map = Wrapping(b),
            Some(Command::MEM_MAP_PALETTE) => self.palette_map = Wrapping(b),
            Some(Command::SET_BORDER_COLOR) =>
                self.border_color_index = b & MASK_INDEX,
            None => return Err(()),
        }
        Ok(0)
    }

    fn tick(&mut self, cpu: &mut Cpu, tick_count: u64) -> TickResult {
        self.backend.tick(cpu, self, tick_count);
        TickResult::Nothing
    }
}

impl<B: Backend> LEM1802<B> {
    pub fn get_screen(&self, cpu: &Cpu) -> Screen {
        let mut screen = [
            Color::default();
            (SCREEN_HEIGHT * SCREEN_WIDTH) as usize
        ];
        for offset in 0..NB_CHARS {
            self.add_char(cpu, &mut screen, offset);
        }
        screen
    }

    fn add_char(&self, cpu: &Cpu, screen: &mut Screen, offset: u16) {
        let video_word = self.get_video_word(cpu, offset);
        let font_item = self.get_font(cpu, video_word.char_idx);
        // x and y are coordinates from top left, but the font items have a different layout so we
        // have to correct it.
        for x in 0..CHAR_WIDTH {
            for y in 0..CHAR_HEIGHT {
                let bit = font_item >> (x * 8 + 7 - y) & 1;
                let mut color = self.get_color(cpu, if bit == 0 {
                    video_word.bg_idx
                } else {
                    video_word.fg_idx
                });
                color.blinking = video_word.blinking;

                let idx = offset + x + (8 * y);
                screen[idx as usize] = color;
            }
        }
    }

    fn get_video_word(&self, cpu: &Cpu, offset: u16) -> VideoWord {
        if self.video_map.0 == 0 {
            unimplemented!()
        } else {
            let idx = self.video_map + Wrapping(offset);
            VideoWord::from_packed(cpu.ram[idx.0 as usize])
        }
    }

    fn get_font(&self, cpu: &Cpu, char_idx: u16) -> u32 {
        if self.font_map.0 == 0 {
            unimplemented!()
        } else {
            let idx = self.font_map + Wrapping(char_idx * 2);
            let w0 = cpu.ram[idx.0 as usize];
            let w1 = cpu.ram[idx.0 as usize + 1];
            (w0 as u32) << 16 & w1 as u32
        }
    }

    fn get_color(&self, cpu: &Cpu, color_idx: u16) -> Color {
        if self.palette_map.0 == 0 {
            unimplemented!()
        } else {
            let idx = self.palette_map + Wrapping(color_idx);
            let color = cpu.ram[idx.0 as usize];
            Color::from_packed(color)
        }
    }
}

struct VideoWord {
    char_idx: u16,
    bg_idx: u16,
    fg_idx: u16,
    blinking: bool,
}

impl VideoWord {
    fn from_packed(w: u16) -> VideoWord {
        VideoWord {
            char_idx: w & MASK_CHAR,
            bg_idx: (w & MASK_COLOR_IDX) >> SHIFT_BG,
            fg_idx: (w & MASK_COLOR_IDX) >> SHIFT_FG,
            blinking: (w & MASK_BLINKING) != 0,
        }
    }
}
