use std::io;
use std::net::SocketAddr;

error_chain!{
    foreign_links {
        IoError(io::Error);
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
