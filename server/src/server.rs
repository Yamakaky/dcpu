use std::sync::mpsc;
use std::net::SocketAddr;

use dcpu::emulator::*;
use event_loop;

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

    pub fn run(mut self) {
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
