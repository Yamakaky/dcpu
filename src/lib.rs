#![doc(html_root_url = "https://yamakaky.github.io/dcpu/")]

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;


#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde")]
extern crate serde;

#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
extern crate num;
#[macro_use]
extern crate glium;
extern crate rustyline;

pub mod assembler;
pub mod emulator;
pub mod iterators;
pub mod types;
