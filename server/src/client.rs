use std::io;
use std::net::SocketAddr;

error_chain!{
    foreign_links {
        io::Error, IoError;
    }
}

pub struct Client {

}

impl Client {
    pub fn connect(addr: SocketAddr) -> Result<Client> {
        try!(Err("pote"))
    }

    pub fn run(self) {

    }
}
