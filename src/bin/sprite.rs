extern crate byteorder;
extern crate dcpu;
extern crate docopt;
extern crate image;
extern crate rustc_serialize;

#[macro_use]
mod utils;

use std::fs::OpenOptions;
use std::io::{self, Write};

use byteorder::WriteBytesExt;
use docopt::Docopt;

const USAGE: &'static str = "
Convert images to LEM1802-compatible blobs.

Usage:
  sprite [options]
  sprite (--help | --version)

Options:
  --font-file <file>     Input black and white font image file.
                         It's a grid with 4x8px characters: 64x64px,
                         32x128px... 4x512px
  --palette-file <file>  Input RGB palette file. At most 16 pixels
                         will be used.
  --image <file>         Image to convert to frame + font + palette.
  --format <format>      Output format to use.
                         Valid values: dat, bin, hex
                         [default: hex]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_font_file: Option<String>,
    flag_palette_file: Option<String>,
    flag_image: Option<String>,
    flag_format: OutputFormat,
}

#[derive(Debug, Copy, Clone, RustcDecodable)]
enum OutputFormat {
    Dat,
    Bin,
    Hex,
}

impl OutputFormat {
    fn to_ext(&self) -> &'static str {
        match *self {
            OutputFormat::Dat => "dat",
            OutputFormat::Bin => "bin",
            OutputFormat::Hex => "hex",
        }
    }
}

fn main_ret() -> i32 {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if let Some(font_path) = args.flag_font_file {
        match image::open(&font_path) {
            Ok(font_img) => {
                match  dcpu::sprite::encode_font(&mut font_img.to_rgb()) {
                    Ok(font) => {
                        let output_path = format!("{}.{}",
                                                  font_path,
                                                  args.flag_format.to_ext());
                        get_it_out(&output_path,
                                   "Font",
                                   &font,
                                   args.flag_format);
                    }
                    Err(e) => die!(1, "{}", e),
                }
            }
            Err(e) => die!(1, "{}", e),
        }
    }
    if let Some(palette_path) = args.flag_palette_file {
        match image::open(&palette_path) {
            Ok(palette_img) => {
                let palette = dcpu::sprite::encode_palette(palette_img.to_rgb()
                                                                      .pixels());
                let output_path = format!("{}.{}",
                                          palette_path,
                                          args.flag_format.to_ext());
                get_it_out(&output_path,
                           "Palette",
                           &palette,
                           args.flag_format);
            }
            Err(e) => die!(1, "{}", e),
        }
    }
    if let Some(image_path) = args.flag_image {
        match image::open(&image_path) {
            Ok(img) => {
                match  dcpu::sprite::encode_image(img.to_rgb()) {
                    Ok((frame, font, palette)) => {
                        let output_path = format!("{}.frame.{}",
                                                  image_path,
                                                  args.flag_format.to_ext());
                        get_it_out(&output_path,
                                   "Image's frame",
                                   &frame,
                                   args.flag_format);
                        let output_path = format!("{}.font.{}",
                                                  image_path,
                                                  args.flag_format.to_ext());
                        get_it_out(&output_path,
                                   "Image's font",
                                   &font,
                                   args.flag_format);
                        let output_path = format!("{}.palette.{}",
                                                  image_path,
                                                  args.flag_format.to_ext());
                        get_it_out(&output_path,
                                   "Font",
                                   &palette,
                                   args.flag_format);
                    }
                    Err(e) => {
                        die!(1, "{}", e);
                    }
                }
            }
            Err(e) => die!(1, "{}", e),
        }
    }
    0
}

fn main() {
    std::process::exit(main_ret());
}

fn get_it_out(path: &str, which: &str, items: &[u16], format: OutputFormat) {
    OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&path)
                .and_then(|mut output| encode_output(&mut output,
                                                     &items,
                                                     format))
                .map(|_| println!("{} written to {}", which, path))
                .unwrap_or_else(|e| println!("Error while opening \"{}\": {}",
                                             path,
                                             e));
}

fn encode_output(output: &mut Write,
                 items: &[u16],
                 format: OutputFormat) -> io::Result<()> {
    match format {
        OutputFormat::Dat => {
            output.write_all(".dat".as_bytes())
                  .and_then(|_| items.iter()
                                     .map(|i|
                                          output.write_fmt(
                                              format_args!(" 0x{:0>4x}", i)
                                          )
                                     ).collect::<io::Result<Vec<()>>>())
                  .and_then(|_| output.write_all("\n".as_bytes()))
        }
        OutputFormat::Bin => {
            items.iter()
                 .map(|i| output.write_u16::<byteorder::LittleEndian>(*i))
                 .collect::<io::Result<Vec<()>>>()
                 .map(|_| ())
        }
        OutputFormat::Hex => {
            items.iter()
                 .map(|i| output.write_fmt(format_args!("0x{:0>4x} ", i)))
                 .collect::<io::Result<Vec<()>>>()
                 .and_then(|_| output.write_all("\n".as_bytes()))
        }
    }
}
