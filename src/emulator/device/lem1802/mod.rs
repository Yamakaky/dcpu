pub mod generic_backend;
mod screen;
#[cfg(feature = "serde")]
mod serde;

use std::any::Any;
use std::fmt::Debug;
use std::num::Wrapping;

use num::traits::FromPrimitive;

use emulator::cpu::Cpu;
use emulator::device::*;
pub use emulator::device::lem1802::screen::*;
use types::Register;

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

pub trait Backend: Debug + Send + Any {
    fn tick<B: Backend>(&self, &Cpu, &LEM1802<B>, tick_count: u64);
    fn hide(&self);
    fn show<B: Backend>(&self, &Cpu, &LEM1802<B>);
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

    fn interrupt(&mut self, cpu: &mut Cpu) -> Result<InterruptDelay> {
        let a = cpu.registers[Register::A];
        let b = cpu.registers[Register::B];
        match try!(Command::from_u16(a)
                           .ok_or(ErrorKind::InvalidCommand(a))) {
            Command::MEM_MAP_SCREEN => {
                self.video_map = Wrapping(b);
                if self.video_map.0 == 0 {
                    self.backend.hide();
                } else {
                    self.backend.show(cpu, self);
                }
            }
            Command::MEM_MAP_FONT => self.font_map = Wrapping(b),
            Command::MEM_MAP_PALETTE => self.palette_map = Wrapping(b),
            Command::SET_BORDER_COLOR =>
                self.border_color_index = b & MASK_INDEX,
        }
        Ok(0)
    }

    fn tick(&mut self, cpu: &mut Cpu, tick_count: u64) -> TickResult {
        self.backend.tick(cpu, self, tick_count);
        TickResult::Nothing
    }

    fn inspect(&self) {
        println!("LEM1802");
        if self.video_map.0 == 0 {
            println!("Currently disabled");
        } else {
            println!("Video ram starts at 0x{:x}", self.video_map.0);
            if self.font_map.0 == 0 {
                println!("Use builtin font");
            } else {
                println!("Font starts at 0x{:x}", self.font_map.0);
            }
            if self.palette_map.0 == 0 {
                println!("Use builtin palette");
            } else {
                println!("Palette starts at 0x{:x}", self.palette_map.0);
            }
            println!("Border color is {:?}",
                     Color::from_packed(self.border_color_index));
        }
    }

    fn as_any(&mut self) -> &mut Any {
        self
    }
}

impl<B: Backend> LEM1802<B> {
    pub fn get_raw_screen(&self, cpu: &Cpu) -> Option<Box<RawScreen>> {
        if self.video_map.0 != 0 {
            let mut raw_screen = Box::new(RawScreen {
                vram: Vram([0; 386]),
                font: Font(self.get_raw_font(cpu)),
                palette: self.get_raw_palette(cpu),
            });
            for (from, to) in cpu.ram
                                 .iter_wrap(self.video_map.0)
                                 .zip(raw_screen.vram.0.iter_mut()) {
                *to = *from;
            }
            Some(raw_screen)
        } else {
            None
        }
    }

    fn get_raw_font(&self, cpu: &Cpu) -> [u16; 256] {
        if self.font_map.0 == 0 {
            DEFAULT_FONT
        } else {
            let mut font = [0; 256];
            for (from, to) in cpu.ram
                                 .iter_wrap(self.font_map.0)
                                 .zip(font.iter_mut()) {
                *to = *from;
            }
            font
        }
    }

    fn get_raw_palette(&self, cpu: &Cpu) -> [u16; 16] {
        if self.palette_map.0 == 0 {
            DEFAULT_PALETTE
        } else {
            let mut palette = [0; 16];
            for (from, to) in cpu.ram
                                 .iter_wrap(self.palette_map.0)
                                 .zip(palette.iter_mut()) {
                *to = *from;
            }
            palette
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
