extern crate getopts;
extern crate SimpleProxy;
use getopts::Options;
use std::env;
use std::net::TcpListener;
use SimpleProxy::client::Tunnel;
use SimpleProxy::client::TunnelReadPort;
use SimpleProxy::client::TunnelWritePort;
use SimpleProxy::client::PortMessage;
use SimpleProxy::socks5::Tcp;
use SimpleProxy::socks5::TcpError;



fn main() {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("s", "server", "server address","S_ADDR");
    opts.reqopt("c", "tunnel-count", "tunnel count", "T_COUNT");
    opts.optopt("l", "listen address", "set listen-address","L_ADDR");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => {
            println!("{}", opts.short_usage(&program));
            return
        }
    };
    let s_addr = matches.opt_str("s").unwrap();
    let tunnel_count = matches.opt_str("c").unwrap();
    //defult listen address 127.0.0.1:1080
    let l_addr = matches.opt_str("l").unwrap_or("127.0.0.1:1080".to_string());
    

}