use std::io;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use bytes::buf::BlockBuf;
use dcpu::emulator;
use futures::{Future, Poll, Async};
use futures::stream::Stream;
use serde_json;
use tokio;
use tokio::io::FramedIo;
use tokio::net::TcpListener;
use tokio::reactor::Core;
use tokio_length_prefix as tlp;
use proto;
use proto::pipeline::Frame;

use msg;
use backends;

error_chain!{
    foreign_links {
        io::Error, IoError;
    }
}

pub enum ServerCommand {
    AddCpu(Box<emulator::Computer>),
}

pub fn start<A: Into<SocketAddr>>(addr: A)
-> Result<mpsc::Receiver<ServerCommand>> {
    let (tx, rx) = mpsc::channel();
    let addr = addr.into();

    thread::spawn(move || {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let serv = TcpListener::bind(&addr, &handle).unwrap();

        let done = serv.incoming().for_each(|(socket, addr)| {
            println!("Incoming connexion from {}", addr);
            try!(socket.set_nodelay(true));
            let framed = tlp::frame::length_prefix_transport(socket);
            let (tx2, rx2) = try!(tokio::channel::channel(&handle));
            let (computer, controler) = backends::new_computer(vec![], tx2);
            let conn = ConnectionLoop {
                socket: framed,
                receiver: rx2,
                sender: tx.clone(),
                controler: controler,
            };
            conn.sender
                .send(ServerCommand::AddCpu(Box::new(computer)))
                .unwrap();
            handle.spawn(conn.map_err(|e| println!("io Error: {}", e)));
            Ok(())
        });
        core.run(done).unwrap();
    });

    Ok(rx)
}

struct ConnectionLoop {
    socket: proto::Framed<tokio::net::TcpStream,
                          tlp::frame::Parser,
                          tlp::frame::Serializer>,
    receiver: tokio::channel::Receiver<msg::ServerMessage>,
    sender: mpsc::Sender<ServerCommand>,
    controler: backends::BackendControler,
}

impl Future for ConnectionLoop {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<(), Error> {
        loop {
            match try_nb!(self.socket.read()) {
                Async::Ready(Frame::Message(msg)) => {
                    match serde_json::from_slice(&msg) {
                        Ok(decoded) => self.controler.dispatch_server(decoded),
                        Err(e) => println!("Decoding error: {}", e),
                    }
                }
                Async::Ready(Frame::Done) => break,
                Async::Ready(r) => println!("{:?}", r),
                Async::NotReady => break,
            }
        }

        loop {
            match try_nb!(self.socket.flush()) {
                Async::Ready(()) => (),
                Async::NotReady => break,
            }

            match try_nb!(self.receiver.poll()) {
                Async::Ready(Some(msg)) => {
                    let encoded = Frame::Message(serde_json::to_vec(&msg)
                                                            .unwrap());
                    if let Err(e) = self.socket
                                        .write(encoded) {
                        println!("Send error: {}", e);
                    }
                }
                _ => break,
            }
        }

        Ok(Async::NotReady)
    }
}
