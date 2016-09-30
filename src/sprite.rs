use image::{self, GenericImage};

const CHAR_HEIGHT: u32 = 8;
const CHAR_WIDTH: u32 = 4;
const NB_CHARS_FONT: u32 = 128;
const NB_PIXELS_FONT_TOTAL: u32 = NB_CHARS_FONT * CHAR_WIDTH * CHAR_HEIGHT;
const FRAME_WIDTH_CHAR: u32 = 32;
const FRAME_HEIGHT_CHAR: u32 = 12;

pub type Frame = [u16; 386];
pub type Font = [u16; 256];
type FontItem = [[bool; CHAR_HEIGHT as usize]; CHAR_WIDTH as usize];
pub type Palette = [u16; 16];

pub fn encode_image(mut img: image::RgbImage) -> Result<(Frame, Font, Palette),
                                                    String> {
    let (img_width, img_heigth) = img.dimensions();
    if img_width != CHAR_WIDTH * FRAME_WIDTH_CHAR ||
       img_heigth != CHAR_HEIGHT * FRAME_HEIGHT_CHAR {
        return Err("The frame image must be 128x96 pixels".into());
    }

    let mut frame_idxs = [(0, 0, 0); 386];
    let mut frame_i = 0;
    let mut font = vec![];
    let mut palette = vec![];
    for y in 0..FRAME_HEIGHT_CHAR {
        for x in 0..FRAME_WIDTH_CHAR {
            let cell = img.sub_image(x * CHAR_WIDTH,
                                     y * CHAR_HEIGHT,
                                     CHAR_WIDTH,
                                     CHAR_HEIGHT);
            let fg_color = cell.get_pixel(0, 0);
            let (font_item, bg_color) = try!(img_to_font_item(&cell, fg_color));
            let id_fg_color =
                insert_vec_id(encode_color(fg_color), &mut palette);
            let id_bg_color =
                insert_vec_id(encode_color(bg_color), &mut palette);
            frame_idxs[frame_i] = if let Some(i) = find_id(font_item, &font) {
                (i, id_fg_color, id_bg_color)
            } else if let Some(i) = find_id(invert_font_item(font_item),
                                            &font) {
                (i, id_bg_color, id_fg_color)
            } else {
                font.push(font_item);
                (font.len() as u16 - 1, id_fg_color, id_bg_color)
            };
            frame_i += 1;
        }
    }
    if font.len() > 128 {
        return Err(format!("The font for the image takes {} items instead of 128", font.len()));
    }
    if palette.len() > 16 {
        return Err(format!("The image uses {} colors instead of 16", palette.len()));
    }

    let mut frame = [0; 386];
    for (&(id_font_item,
           id_color_fg,
           id_color_bg), to) in frame_idxs.iter().zip(frame.iter_mut()) {
        *to = encode_char(id_font_item, id_color_fg, id_color_bg);
    }
    let mut palette_array = [0; 16];
    palette_array[..palette.len()].copy_from_slice(&palette);
    Ok((frame, encode_font_items(&font), palette_array))
}

fn invert_font_item(item: FontItem) -> FontItem {
    let mut inverted: FontItem = Default::default();
    for x in 0..(CHAR_WIDTH as usize) {
        for y in 0..(CHAR_HEIGHT as usize) {
            inverted[x][y] = !item[x][y];
        }
    }
    inverted
}

fn insert_vec_id<T: PartialEq<T>>(item: T, items: &mut Vec<T>) -> u16 {
    for (i, c) in items.iter().enumerate() {
        if *c == item {
            return i as u16;
        }
    }
    items.push(item);
    (items.len() - 1) as u16
}

fn find_id<T: PartialEq<T>>(item: T, items: &[T]) -> Option<u16> {
    for (i, c) in items.iter().enumerate() {
        if *c == item {
            return Some(i as u16);
        }
    }
    None
}

fn encode_char(id_font_item: u16, id_color_fg: u16, id_color_bg: u16) -> u16 {
    id_color_fg << 12 | id_color_bg << 8 | id_font_item
}

pub fn encode_font(img: &mut image::RgbImage) -> Result<[u16; 256], String> {
    let (img_width, img_heigth) = img.dimensions();
    if img_width * img_heigth != NB_PIXELS_FONT_TOTAL ||
       img_width % CHAR_WIDTH != 0 ||
       img_heigth % CHAR_HEIGHT != 0 {
        return Err("The font image must be rectangular, with x and y multiples \
of 4 and 8 respectively, like 64*64px, 32x128px...".into());
    }

    let mut font_items = vec![];
    for x in 0..(img_width / CHAR_WIDTH) {
        for y in 0..(img_heigth / CHAR_HEIGHT) {
            let (font_item, _) =
                try!(img_to_font_item(&img.sub_image(x * CHAR_WIDTH,
                                                     y * CHAR_HEIGHT,
                                                     CHAR_WIDTH,
                                                     CHAR_HEIGHT),
                                      image::Rgb { data: [0, 0, 0]}));
            font_items.push(font_item);
        }
    }
    Ok(encode_font_items(&font_items))
}

fn img_to_font_item(img: &image::SubImage<image::ImageBuffer<image::Rgb<u8>,
                                                             Vec<u8>>>,
                        fg_color: image::Rgb<u8>)
    -> Result<(FontItem, image::Rgb<u8>), String> {
    assert_eq!(img.dimensions(), (CHAR_WIDTH, CHAR_HEIGHT));
    let mut item: FontItem = Default::default();
    let mut maybe_bg_color: Option<image::Rgb<u8>> = None;
    for x in 0..CHAR_WIDTH {
        for y in 0..CHAR_HEIGHT {
            let pixel = img.get_pixel(x, y);
            item[x as usize][y as usize] = if pixel == fg_color {
                true
            } else {
                if let Some(bg_color) = maybe_bg_color {
                    if pixel != bg_color {
                        return Err("Each char must have 2 colors max".into());
                    }
                } else {
                    maybe_bg_color = Some(pixel);
                }
                false
            }
        }
    }
    Ok((item, maybe_bg_color.unwrap_or(fg_color)))
}

fn encode_font_items(items: &[FontItem]) -> Font {
    let mut font = [0; 256];
    for (i, item) in items.iter().take(128).enumerate() {
        let (l, r) = encode_font_item(item);
        font[2 * i] = l;
        font[2 * i + 1] = r;
    }
    font
}

fn encode_font_item(item: &FontItem) -> (u16, u16) {
    let mut l = 0;
    let mut r = 0;
    for x in 0..CHAR_WIDTH {
        for y in 0..CHAR_HEIGHT {
            let bit = item[x as usize][y as usize] as u16;
            let rel_x = x % 2;
            let shift = rel_x * (CHAR_HEIGHT) + 7 - y;
            if x % CHAR_WIDTH < 2 {
                l |= bit << (15 - shift);
            } else {
                r |= bit << (15 - shift);
            }
        }
    }
    (l, r)
}

pub fn encode_palette<'a, It>(colors: It) -> [u16; 16]
    where It: IntoIterator<Item=&'a image::Rgb<u8>> {

    let mut palette = [0; 16];
    for (color, encoded) in colors.into_iter().zip(palette.iter_mut()) {
        *encoded = encode_color(*color);
    }
    palette
}

fn encode_color(color: image::Rgb<u8>) -> u16 {
    let r = (color.data[0] / 16) as u16;
    let g = (color.data[1] / 16) as u16;
    let b = (color.data[2] / 16) as u16;
    r << 8 | g << 4 | b
}
