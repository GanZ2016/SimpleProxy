extern crate getopts;
extern crate SimpleProxy;
use getopts::Options;
use std::env;
use std::net::TcpListener;
use SimpleProxy::server::Tunnel;

fn main()  {
    // get and check commands
    // Ref: https://doc.rust-lang.org/getopts/getopts/index.html
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    // pub fn reqopt(&mut self, short_name: &str, long_name: &str, desc: &str, hint: &str)
    //                          -> &mut Options
    opts.reqopt("l", "listen address", "set listen-address","L_ADDR");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { 
            print!("l");
            return 
            }
    };
    matches.opt_str("l").expect("wrong address");
    let l_addr = matches.opt_str("l").unwrap();
    // Creates a new `TcpListener` which will be bound to the specified
    // address.
    let listener = TcpListener::bind(&l_addr[..]).unwrap();
    // Returns an iterator over the connections being received on this
    // listener.
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client!");
                Tunnel::new(stream);
            },
            Err(_) => {println!("connection failed!");}
        }
    }
}