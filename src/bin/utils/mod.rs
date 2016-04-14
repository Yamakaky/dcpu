use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use byteorder;
use byteorder::ReadBytesExt;

#[allow(dead_code)]
#[derive(Debug)]
pub struct IterU16<I> {
    pub input: I
}

impl<I: ReadBytesExt> Iterator for IterU16<I> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        self.input.read_u16::<byteorder::BigEndian>().ok()
    }
}

#[allow(dead_code)]
pub fn get_input(i: Option<String>) -> Box<Read> {
    if let Some(path) = i {
        Box::new(File::open(path).expect("Open file error"))
    } else {
        Box::new(::std::io::stdin())
    }
}

#[allow(dead_code)]
pub fn get_output(o: Option<String>) -> Box<Write> {
    if let Some(path) = o {
        Box::new(OpenOptions::new()
                             .write(true)
                             .create(true)
                             .open(path)
                             .expect("Open file error"))
    } else {
        Box::new(::std::io::stdout())
    }
}

macro_rules! die {
    ( $exit:expr, $($x:expr),* ) => (
        {
            let mut stderr = ::std::io::stderr();
            writeln!(stderr, $($x),*).unwrap();
            return $exit;
        }
    )
}
