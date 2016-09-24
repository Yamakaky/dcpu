extern crate byteorder;
extern crate dcpu;
extern crate docopt;
extern crate image;
extern crate rustc_serialize;

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
  --font-file <file>     Input black and white font file.
                         It's a 64x64px, 16x8 8x4px characters image.
  --palette-file <file>  Input RGB palette file. At most 16 pixels
                         will be used.
  --format <format>      Output format to use.
                         Valid values: dat, bin, hex
                         [default: hex]
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_font_file: Option<String>,
    flag_palette_file: Option<String>,
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

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if let Some(font_path) = args.flag_font_file {
        match image::open(&font_path) {
            Ok(font_img) => {
                let font = dcpu::sprite::encode_font(font_img.to_rgb());
                let output_path = format!("{}.{}",
                                          font_path,
                                          args.flag_format.to_ext());
                let mut output = OpenOptions::new()
                                             .write(true)
                                             .truncate(true)
                                             .create(true)
                                             .open(&output_path)
                                             .unwrap();
                encode_output(&mut output, &font, args.flag_format).unwrap();
                println!("Font written to {}", output_path);
            }
            Err(e) => println!("{:?}", e),
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
                let mut output = OpenOptions::new()
                                             .write(true)
                                             .truncate(true)
                                             .create(true)
                                             .open(&output_path)
                                             .unwrap();
                encode_output(&mut output, &palette, args.flag_format).unwrap();
                println!("Palette written to {}", output_path);
            }
            Err(e) => println!("{:?}", e),
        }
    }
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
