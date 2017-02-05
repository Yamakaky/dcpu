use std::fmt;

pub const MASK_INDEX: u16 = 0xf;
pub const SCREEN_HEIGHT: u16 = 96;
pub const SCREEN_WIDTH: u16 = 128;
pub const SCREEN_SIZE: u16 = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const CHAR_HEIGHT: u16 = 8;
pub const CHAR_WIDTH: u16 = 4;
pub const CHAR_SIZE: u16 = CHAR_HEIGHT * CHAR_WIDTH;
pub const NB_CHARS: u16 = 32 * 12;

pub const MASK_BLINKING: u16 = 1 << 7;
pub const MASK_COLOR_IDX: u16 = 0xf;
pub const MASK_CHAR: u16 = 0x7f;
pub const SHIFT_FG: u16 = 12;
pub const SHIFT_BG: u16 = 8;

/// Wrappers for easier serde implementation
pub struct Vram(pub [u16; 386]);
pub struct Font(pub [u16; 256]);

#[derive(Serialize, Deserialize)]
pub struct RawScreen {
    pub vram: Vram,
    pub font: Font,
    pub palette: [u16; 16],
}

impl fmt::Debug for RawScreen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A beautiful raw screen with shiny pixels")
    }
}

impl From<Box<RawScreen>> for Box<Screen> {
    fn from(raw: Box<RawScreen>) -> Box<Screen> {
        let mut screen =
            Box::new(Screen([Color::default(); SCREEN_SIZE as usize]));
        for offset in 0..NB_CHARS {
            raw.add_char(&mut screen, offset);
        }
        screen
    }
}

impl RawScreen {
    pub fn add_char(&self, screen: &mut Screen, char_offset: u16) {
        let video_word = self.get_video_word(char_offset);
        let font_item = self.get_font(video_word.char_idx);
        // x and y are coordinates from top left, but the font items have a different layout so we
        // have to correct it.
        for x in 0..CHAR_WIDTH {
            for y in 0..CHAR_HEIGHT {
                let bit = (font_item >> (x * CHAR_HEIGHT + 7 - y)) & 1;
                let mut color = self.get_color(if bit == 0 {
                    video_word.bg_idx
                } else {
                    video_word.fg_idx
                });
                color.blinking = video_word.blinking;

                let byte_offset = (char_offset / 32) * (CHAR_SIZE * 32)
                                + (char_offset % 32) * CHAR_WIDTH;
                let idx = byte_offset
                        + (CHAR_WIDTH - x - 1)
                        + (SCREEN_WIDTH * (CHAR_HEIGHT - y - 1));
                screen.0[idx as usize] = color;
            }
        }
    }

    fn get_video_word(&self, char_offset: u16) -> VideoWord {
        VideoWord::from_packed(self.vram.0[char_offset as usize])
    }

    fn get_font(&self, char_idx: u16) -> u32 {
        let (w0, w1) =
            (self.font.0[char_idx as usize * 2],
             self.font.0[char_idx as usize * 2 + 1]);
        (w0 as u32) << 16 | w1 as u32
    }

    fn get_color(&self, color_idx: u16) -> Color {
        Color::from_packed(self.palette[color_idx as usize])
    }
}

pub struct Screen(pub [Color; SCREEN_SIZE as usize]);

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A beautiful screen with shiny pixels")
    }
}

#[derive(Serialize, Deserialize)]
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
            bg_idx: (w >> SHIFT_BG) & MASK_COLOR_IDX,
            fg_idx: (w >> SHIFT_FG) & MASK_COLOR_IDX,
            blinking: (w & MASK_BLINKING) != 0,
        }
    }
}
