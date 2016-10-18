use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};

#[allow(dead_code)]
pub fn get_input(i: Option<String>) -> Result<Box<Read>, io::Error> {
    if let Some(path) = i {
        match File::open(path) {
            Ok(f) => Ok(Box::new(BufReader::new(f))),
            Err(e) => Err(e),
        }
    } else {
        Ok(Box::new(::std::io::stdin()))
    }
}

#[allow(dead_code)]
pub fn get_output(o: Option<String>) -> Result<Box<Write>, io::Error> {
    if let Some(path) = o {
        match OpenOptions::new()
                          .write(true)
                          .create(true)
                          .truncate(true)
                          .open(path) {
            Ok(f) => Ok(Box::new(BufWriter::new(f))),
            Err(e) => Err(e),
        }
    } else {
        Ok(Box::new(::std::io::stdout()))
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
