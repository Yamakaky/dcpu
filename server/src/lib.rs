#![feature(proc_macro)]

extern crate bytes;
extern crate dcpu;
#[macro_use]
extern crate error_chain;
extern crate rayon;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate tokio_core as tokio;
extern crate tokio_proto as proto;
extern crate tokio_length_prefix;
extern crate futures;

mod backends;
mod client;
mod event_loop;
mod msg;
mod server;

pub use client::Client;
pub use server::Server;
