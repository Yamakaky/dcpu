#![doc(html_root_url = "https://yamakaky.github.io/dcpu/")]

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;


#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "clap")]
#[macro_use]
extern crate clap;
#[cfg(feature = "clap")]
extern crate colored;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
#[cfg(feature = "glium")]
#[macro_use]
extern crate glium;
#[cfg(feature = "rustyline")]
extern crate rustyline;
extern crate signal;

pub mod assembler;
#[cfg(not(crate_type = "rlib"))]
pub mod c_api;
pub mod byteorder;
pub mod emulator;
pub mod iterators;
pub mod types;
