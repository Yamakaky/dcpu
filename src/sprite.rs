use image;

const CHAR_HEIGHT: u32 = 8;
const CHAR_WIDTH: u32 = 4;
const NB_CHARS_HEIGHT: u32 = 8;
const NB_CHARS_WIDTH: u32 = 16;
const FILE_HEIGHT: u32 = CHAR_HEIGHT * NB_CHARS_HEIGHT;
const FILE_WIDTH: u32 = CHAR_WIDTH * NB_CHARS_WIDTH;

pub fn encode_font(img: image::RgbImage) -> Result<[u16; 256], String> {
    if img.dimensions() != (FILE_WIDTH, FILE_HEIGHT) {
        return Err("The font image must be 64x64px".into());
    }

    let mut font = [0u16; 256];
    for (x, y, pixel) in img.enumerate_pixels() {
        let bit = if pixel.data == [0, 0, 0] {
            1
        } else if pixel.data == [255, 255, 255] {
            0
        } else {
            return Err(
                format!("Invalid pixel at ({}, {}), should be black or white",
                        x, y)
            );
        };
        let char_id = (x / CHAR_WIDTH) + (y / CHAR_HEIGHT) * NB_CHARS_WIDTH;
        let char_rel_x = x % 2;
        let char_rel_y = y % CHAR_HEIGHT;
        let shift = char_rel_x * (CHAR_HEIGHT) + 7 - char_rel_y;
        font[2 * char_id as usize + if x % CHAR_WIDTH < 2 {0} else {1}]
            |= bit << (15 - shift);
    }
    Ok(font)
}

pub fn encode_palette<'a, It>(colors: It) -> [u16; 16]
    where It: IntoIterator<Item=&'a image::Rgb<u8>> {

    let mut palette = [0; 16];
    for (color, encoded) in colors.into_iter().zip(palette.iter_mut()) {
        let r = (color.data[0] / 16) as u16;
        let g = (color.data[1] / 16) as u16;
        let b = (color.data[2] / 16) as u16;
        *encoded = r << 8 | g << 4 | b;
    }
    palette
}

