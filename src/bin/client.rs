extern crate getopts;
extern crate SimpleProxy;
use getopts::Options;
use std::env;
use std::thread;
use std::net::{TcpListener,TcpStream};
use std::str::from_utf8;
use SimpleProxy::client::Tunnel;
use SimpleProxy::client::TunnelReadPort;
use SimpleProxy::client::TunnelWritePort;
use SimpleProxy::client::PortMessage;
use SimpleProxy::client::tunnel_write_port;

use SimpleProxy::socks5::Tcp;
use SimpleProxy::socks5::TcpError;


fn start_tunnels(l_addr: String, s_addr:String,tunnel_count:u32) {
    //tunnel vector for multi-tunnels
    let mut tunnels = Vec::new();
    for i in 0..tunnel_count {
        let new_tunnel = Tunnel::new(i,s_addr.clone());
        tunnels.push(new_tunnel);
    }
    let mut index = 0;
    let new_listener = TcpListener::bind(l_addr.as_str()).unwrap();
    println!("connecting to {}", l_addr.as_str());
    // Returns an iterator over the connections being received on this listener.
    for stream in new_listener.incoming() {
        match stream {
            Ok(stream) => {
                {
                    let new_tunnel: &mut Tunnel = tunnels.get_mut(index).unwrap();
                    let (write_port, read_port) = new_tunnel.open_port();
                    thread::spawn(move || {
                        tunnel_write_port(stream,write_port,read_port);
                    });
                }
                index = (index+1)%tunnels.len();
            },
            Err(_) => {},
        }
    }

}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("s", "server", "server address","server_address:port");
    opts.reqopt("c", "tunnel-count", "tunnel count", "tunnel_count");
    opts.optopt("l", "listen address", "set listen-address","defult 127.0.0.1:1080");
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
    
    let count :u32 = match tunnel_count.parse(){
        Err(_) | Ok(0) =>{
            println!("count must larger than 0");
            return

        },
        Ok(count) =>{count},
    };  
    start_tunnels(l_addr,s_addr,count);
}