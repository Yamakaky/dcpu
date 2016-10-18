extern crate dcpu_server;

fn main() {
    let addr = "127.0.0.1:1245".parse().unwrap();
    let mut client = dcpu_server::Client::connect(addr).unwrap();
    client.run();
}
