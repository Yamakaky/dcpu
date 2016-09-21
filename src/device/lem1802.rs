use std::fmt::Debug;
use std::num::Wrapping;

use num::traits::FromPrimitive;

use cpu::Cpu;
use device::*;

const MASK_INDEX: u16 = 0xf;
pub const SCREEN_HEIGHT: u16 = 96;
pub const SCREEN_WIDTH: u16 = 128;
pub const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
const CHAR_HEIGHT: u16 = 8;
const CHAR_WIDTH: u16 = 4;
const CHAR_SIZE: u16 = CHAR_HEIGHT * CHAR_WIDTH;
const NB_CHARS: u16 = 32 * 12;

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
pub type Screen = [Color; SCREEN_SIZE as usize];

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
    pub fn get_screen(&self, cpu: &Cpu) -> Box<Screen> {
        // Stack overflow if we don't use Box
        let mut screen = Box::new([Color::default(); SCREEN_SIZE as usize]);
        for offset in 0..NB_CHARS {
            self.add_char(cpu, &mut screen, offset);
        }
        screen
    }

    fn add_char(&self, cpu: &Cpu, screen: &mut Screen, char_offset: u16) {
        let video_word = self.get_video_word(cpu, char_offset);
        let font_item = self.get_font(cpu, video_word.char_idx);
        // x and y are coordinates from top left, but the font items have a different layout so we
        // have to correct it.
        for x in 0..CHAR_WIDTH {
            for y in 0..CHAR_HEIGHT {
                let bit = (font_item >> (x * CHAR_HEIGHT + 7 - y)) & 1;
                let mut color = self.get_color(cpu, if bit == 0 {
                    video_word.bg_idx
                } else {
                    video_word.fg_idx
                });
                color.blinking = video_word.blinking;

                let idx = char_offset * CHAR_SIZE + x + (SCREEN_WIDTH * y);
                println!("{} {} {} {}", char_offset, y, x, idx);
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
        let (w0, w1) = if self.font_map.0 == 0 {
            (DEFAULT_FONT[char_idx as usize], DEFAULT_FONT[char_idx as usize + 1])
        } else {
            let idx = self.font_map + Wrapping(char_idx * 2);
            (cpu.ram[idx.0 as usize], cpu.ram[idx.0 as usize + 1])
        };
        (w0 as u32) << 16 & w1 as u32
    }

    fn get_color(&self, cpu: &Cpu, color_idx: u16) -> Color {
        if self.palette_map.0 == 0 {
            Color::from_packed(DEFAULT_PALETTE[color_idx as usize])
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

// Taken from
// https://github.com/azertyfun/DCPU-Toolchain/blob/master/src/tk/azertyfun/dcputoolchain/emulator/LEM1802.java
const DEFAULT_FONT: [u16; 256] = [
    0x000F, 0x0808, 0x080F, 0x0808, 0x08F8, 0x0808, 0x00FF, 0x0808,
    0x0808, 0x0808, 0x08FF, 0x0808, 0x00FF, 0x1414, 0xFF00, 0xFF08,
    0x1F10, 0x1714, 0xFC04, 0xF414, 0x1710, 0x1714, 0xF404, 0xF414,
    0xFF00, 0xF714, 0x1414, 0x1414, 0xF700, 0xF714, 0x1417, 0x1414,
    0x0F08, 0x0F08, 0x14F4, 0x1414, 0xF808, 0xF808, 0x0F08, 0x0F08,
    0x001F, 0x1414, 0x00FC, 0x1414, 0xF808, 0xF808, 0xFF08, 0xFF08,
    0x14FF, 0x1414, 0x080F, 0x0000, 0x00F8, 0x0808, 0xFFFF, 0xFFFF,
    0xF0F0, 0xF0F0, 0xFFFF, 0x0000, 0x0000, 0xFFFF, 0x0F0F, 0x0F0F,
    0x0000, 0x0000, 0x005f, 0x0000, 0x0300, 0x0300, 0x3e14, 0x3e00,
    0x266b, 0x3200, 0x611c, 0x4300, 0x3629, 0x7650, 0x0002, 0x0100,
    0x1c22, 0x4100, 0x4122, 0x1c00, 0x1408, 0x1400, 0x081c, 0x0800,
    0x4020, 0x0000, 0x0808, 0x0800, 0x0040, 0x0000, 0x601c, 0x0300,
    0x3e49, 0x3e00, 0x427f, 0x4000, 0x6259, 0x4600, 0x2249, 0x3600,
    0x0f08, 0x7f00, 0x2745, 0x3900, 0x3e49, 0x3200, 0x6119, 0x0700,
    0x3649, 0x3600, 0x2649, 0x3e00, 0x0024, 0x0000, 0x4024, 0x0000,
    0x0814, 0x2200, 0x1414, 0x1400, 0x2214, 0x0800, 0x0259, 0x0600,
    0x3e59, 0x5e00, 0x7e09, 0x7e00, 0x7f49, 0x3600, 0x3e41, 0x2200,
    0x7f41, 0x3e00, 0x7f49, 0x4100, 0x7f09, 0x0100, 0x3e41, 0x7a00,
    0x7f08, 0x7f00, 0x417f, 0x4100, 0x2040, 0x3f00, 0x7f08, 0x7700,
    0x7f40, 0x4000, 0x7f06, 0x7f00, 0x7f01, 0x7e00, 0x3e41, 0x3e00,
    0x7f09, 0x0600, 0x3e61, 0x7e00, 0x7f09, 0x7600, 0x2649, 0x3200,
    0x017f, 0x0100, 0x3f40, 0x7f00, 0x1f60, 0x1f00, 0x7f30, 0x7f00,
    0x7708, 0x7700, 0x0778, 0x0700, 0x7149, 0x4700, 0x007f, 0x4100,
    0x031c, 0x6000, 0x417f, 0x0000, 0x0201, 0x0200, 0x8080, 0x8000,
    0x0001, 0x0200, 0x2454, 0x7800, 0x7f44, 0x3800, 0x3844, 0x2800,
    0x3844, 0x7f00, 0x3854, 0x5800, 0x087e, 0x0900, 0x4854, 0x3c00,
    0x7f04, 0x7800, 0x047d, 0x0000, 0x2040, 0x3d00, 0x7f10, 0x6c00,
    0x017f, 0x0000, 0x7c18, 0x7c00, 0x7c04, 0x7800, 0x3844, 0x3800,
    0x7c14, 0x0800, 0x0814, 0x7c00, 0x7c04, 0x0800, 0x4854, 0x2400,
    0x043e, 0x4400, 0x3c40, 0x7c00, 0x1c60, 0x1c00, 0x7c30, 0x7c00,
    0x6c10, 0x6c00, 0x4c50, 0x3c00, 0x6454, 0x4c00, 0x0836, 0x4100,
    0x0077, 0x0000, 0x4136, 0x0800, 0x0201, 0x0201, 0x0205, 0x0200,
];

// Taken from
// https://github.com/azertyfun/DCPU-Toolchain/blob/master/src/tk/azertyfun/dcputoolchain/emulator/LEM1802.java
const DEFAULT_PALETTE: [u16; 16] = [
    0x000, 0x00a, 0x0a0, 0x0aa, 0xa00, 0xa0a, 0xa50, 0xaaa,
    0x555, 0x55f, 0x5f5, 0x5ff, 0xf55, 0xf5f, 0xff5, 0xfff,
];
