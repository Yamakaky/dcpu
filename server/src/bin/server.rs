extern crate dcpu_server;

fn main() {
    let addr = "127.0.0.1:1245".parse().unwrap();
    let mut server = dcpu_server::Server::start(addr).unwrap();
    server.run();
}
