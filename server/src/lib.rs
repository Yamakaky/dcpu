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
mod event_loop;
mod msg;

use std::sync::mpsc;
use std::net::SocketAddr;

use dcpu::emulator::*;

pub struct Server {
    dcpus: Vec<Computer>,
    commands: mpsc::Receiver<event_loop::ServerCommand>,
}

impl Server {
    pub fn start(addr: SocketAddr) -> event_loop::Result<Server> {
        Ok(Server {
            dcpus: Vec::default(),
            commands: try!(event_loop::start(addr)),
        })
    }

    pub fn run(&mut self) {
        //tick_rec(&mut self.dcpus);
        loop {
            'cmds: loop {
                match self.commands.try_recv() {
                    Ok(cmd) => match cmd {
                        event_loop::ServerCommand::AddCpu(c) => self.dcpus.push(*c),
                    },
                    Err(mpsc::TryRecvError::Disconnected) => unimplemented!(),
                    Err(mpsc::TryRecvError::Empty) => break 'cmds,
                }
            }

            for dcpu in &mut self.dcpus {
                dcpu.tick().unwrap();
            }
        }
    }
}

//fn tick_rec(cs: &mut [Computer]) {
//    let size = cs.len();
//    if size <= 1 {
//        for _ in 0..1000 {
//            for c in cs.iter_mut() {
//                c.tick().unwrap();
//            }
//        }
//    } else {
//        let (low, high) = cs.split_at_mut(size / 2);
//        rayon::join(|| tick_rec(low), || tick_rec(high));
//    }
//}
