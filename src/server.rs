use std::sync::mpsc::{sync_channel, SyncSender, channel, Sender, Receiver};
use std::thread;
use std::vec::Vec;
use std::str::from_utf8;
use std::collections::HashMap;
use std::net::{TcpStream,SocketAddr,lookup_host};
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError};
use super::protocol::{
    cs, sc,
    HEARTBEAT_INTERVAL_MS,
    ALIVE_TIMEOUT_TIME_MS
};
use std::io::Write;


// Enumeration Type of Message Transfer in Tunnel
enum Message {
    CSHeartbeat,
    CSOpenPort(u32),
    CSClosePort(u32),
    CSShutdownWrite(u32),
    CSConnectDN(u32, Vec<u8>, u16),
    CSData(u8, u32, Vec<u8>),

    SCClosePort(u32),
    SCShutdownWrite(u32),
    SCConnectOk(u32, Vec<u8>),
    SCData(u32, Vec<u8>),

    PortDrop(u32),

    CloseTunnel,
}
//Enumeration Type for Port Message.
pub enum PortMessage {

    ConnectDN(Vec<u8>, u16),
    Data(u8, Vec<u8>),
    ShutdownWrite,
    ClosePort,
}


pub struct Tunnel;


pub struct TunnelWritePort{
    port_id:u32,
    tx:SyncSender<Message>,
}

pub struct TunnelReadPort{
    port_id:u32,
    tx:SyncSender<Message>,
    rx:Receiver<PortMessage>,
}
struct PortMapValue {

    count: u32,
    tx: Sender<PortMessage>,
}

type PortMap = HashMap<u32, PortMapValue>;

impl Tunnel {
    pub fn new(stream: TcpStream) {
        thread::spawn(move || {
            tunnel_core_task(stream);
        });
    } 
}
impl Copy for Tunnel {
}

impl Clone for Tunnel {
    fn clone(&self) -> Self {
        *self
    }
}


impl TunnelWritePort {
    fn connect_ok(&self, buf: Vec<u8>) {
        match self.tx.send(Message::SCConnectOk(self.port_id, buf)){
            Ok(_) => {},
            Err(_) => {}
        };
    }

    fn write(&self, buf: Vec<u8>) {
        match self.tx.send(Message::SCData(self.port_id, buf)){
            Ok(_) => {},
            Err(_) => {}
        };
    }

    fn shutdown_write(&self) {
        match self.tx.send(Message::SCShutdownWrite(self.port_id)){
            Ok(_) => {},
            Err(_) => {}
        };
    }

    fn close(&self) {
        match self.tx.send(Message::SCClosePort(self.port_id)){
            Ok(_) => {},
            Err(_) => {}
        };
    }

}
impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        match self.tx.send(Message::PortDrop(self.port_id)){
            Ok(_) => {},
            Err(_) => {}
        };
    }
}

impl TunnelReadPort {
    pub fn read(&self) -> PortMessage {
        self.rx.recv().unwrap()
    }
}

impl Drop for TunnelReadPort {
    fn drop(&mut self) {
        match self.tx.send(Message::PortDrop(self.port_id)){
            Ok(_) => {},
            Err(_) => {}
        };
    }
}

fn tunnel_port_write(s: TcpStream, write_port: TunnelWritePort) {
    let mut stream = Tcp::new(s);

    loop {
        match stream.read_at_most(1024) {
            Ok(buf) => {
                write_port.write(buf);
            },
            Err(TcpError::Eof) => {
                stream.shutdown_read();
                write_port.shutdown_write();
                break
            },
            Err(_) => {
                stream.shutdown();
                write_port.close();
                break
            }
        }
    }
}

fn tunnel_port_read(s: TcpStream, read_port: TunnelReadPort) {
    let mut stream = Tcp::new(s);

    loop {
        match read_port.read() {
            PortMessage::Data(cs::DATA, buf) => {
                match stream.write(&buf[..]) {
                    Ok(_) => {},
                    Err(_) => {
                        stream.shutdown();
                        break
                    }
                }
            },
            PortMessage::ShutdownWrite => {
                stream.shutdown_write();
                break
            },
            _ => {
                stream.shutdown();
                break
            }
        }
    }
}
fn tunnel_port_task(read_port: TunnelReadPort, write_port: TunnelWritePort) {
    let os = match read_port.read() {
        PortMessage::Data(cs::CONNECT, buf) => {
            TcpStream::connect(from_utf8(&buf[..]).unwrap()).ok()
        },
        PortMessage::ConnectDN(domain_name, port) => {
            match lookup_host(from_utf8(&domain_name[..]).unwrap()) {
                Ok(hosts) => {
                    let mut stream = None;
                    for host in hosts {
                        let conn = match host {
                            SocketAddr::V4(addr_v4) =>
                                TcpStream::connect((addr_v4.ip().clone(), port)),
                        };
                        match conn {
                            Ok(s) => { stream = Some(s); break; },
                            Err(_) => {}
                        }
                    }
                    stream
                },
                Err(_) => None
            }
        },
        _ => None
    };

    let s = match os {
        Some(s) => s,
        None => {
            return write_port.close();
        }
    };

    match s.local_addr() {
        Ok(addr) => {
            let mut buf = Vec::new();
            match write!(&mut buf, "{}", addr){
                Ok(_) => {},
                Err(_) => {}
            };
            write_port.connect_ok(buf);
        },
        Err(_) => {
            return write_port.close();
        }
    }

    let receiver = s.try_clone().unwrap();
    thread::spawn(move || {
        tunnel_port_write(receiver, write_port);
    });

    tunnel_port_read(s, read_port);
}
fn tunnel_tcp_recv(receiver: TcpStream,
                   core_tx: SyncSender<Message>) {
    let mut stream = Tcp::new(receiver);
    match tunnel_recv_loop(&core_tx, &mut stream){
        Ok(_) => {},
        Err(_) => {}
    };
    stream.shutdown();
    match core_tx.send(Message::CloseTunnel){
        Ok(_) => {},
        Err(_) => {}
    };
}

fn tunnel_recv_loop(core_tx: &SyncSender<Message>,
                    stream: &mut Tcp) -> Result<(), TcpError> {


    loop {
        let op = try!(stream.read_u8());
        if op == cs::HEARTBEAT {
            match core_tx.send(Message::CSHeartbeat){
                Ok(_) => {},
                Err(_) => {}
            };
            continue
        }

        let id = try!(stream.read_u32());
        match op {
            cs::OPEN_PORT => {
                match core_tx.send(Message::CSOpenPort(id)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },
            cs::CLOSE_PORT => {
                match core_tx.send(Message::CSClosePort(id)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },
            cs::SHUTDOWN_WRITE => {
                match core_tx.send(Message::CSShutdownWrite(id)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            cs::CONNECT_DOMAIN_NAME => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_size((len - 2) as usize));
                let port = try!(stream.read_u16());
                let domain_name = buf.clone();
                match core_tx.send(Message::CSConnectDN(id, domain_name, port)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            _ => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_size(len as usize));
                match core_tx.send(Message::CSData(op, id, buf)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            }
        }
    }
}


pub fn tunnel_core_task(sender: TcpStream){
    // Creates a new synchronous, bounded channel, with buffer size 10000.
    // tx is the sending half (tx for transmission), and rx is the receiving
    // half (rx for receiving).
    let (core_tx, core_rx) = sync_channel(10000);
    let receiver = sender.try_clone().unwrap();
    let core_tx2 = core_tx.clone();

    thread::spawn(move || {
        tunnel_tcp_recv(receiver, core_tx2);
    });

    let mut stream = Tcp::new(sender);
    let mut port_map = PortMap::new();

    match tunnel_loop(&core_tx, &core_rx, &mut stream, &mut port_map){
        Ok(_) => {},
        Err(_) => {}
    };

    stream.shutdown();
    for (_, value) in port_map.iter() {
        match value.tx.send(PortMessage::ClosePort){
            Ok(_) => {},
            Err(_) => {}
        };
    }
}
/// SyncSender<Message>
/// The sending-half of Rust's synchronous channel type. This half can only be
/// owned by one thread, but it can be cloned to send to other threads.

/// Receiver<Message>
/// The receiving-half of Rust's channel type. This half can only be owned by
/// one thread
fn tunnel_loop(core_tx: &SyncSender<Message>,
               core_rx: &Receiver<Message>, stream: &mut Tcp,
               port_map: &mut PortMap) -> Result<(), TcpError> 
    {
    let timer = Timer::new(HEARTBEAT_INTERVAL_MS);
    let mut alive_time = time::get_time();

    loop {
        select! {
            _ = timer.recv() => {
                let duration = time::get_time() - alive_time;
                if duration.num_milliseconds() > ALIVE_TIMEOUT_TIME_MS {
                    break
                }
            },

            msg = core_rx.recv() => match msg.unwrap() {
            Message::CSHeartbeat => {
                alive_time = time::get_time();
                try!(stream.write_u8(sc::HEARTBEAT_RSP));
            },

            Message::CSOpenPort(id) => {
                    alive_time = time::get_time();
        /// Creates a new asynchronous channel, returning the sender/receiver halves.
        /// All data sent on the sender will become available on the receiver, and no
        /// send will block the calling thread (this channel has an "infinite buffer").         
                    let (tx, rx) = channel();
                    port_map.insert(id, PortMapValue { count: 2, tx: tx });

                    let read_port = TunnelReadPort {
                        port_id: id, tx: core_tx.clone(), rx: rx
                    };
                    let write_port = TunnelWritePort {
                        port_id: id, tx: core_tx.clone()
                    };

                    thread::spawn(move || {
                        tunnel_port_task(read_port, write_port);
                    });
                },

                Message::CSClosePort(id) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                        match value.tx.send(PortMessage::ClosePort){
                            Ok(_) => {},
                            Err(_) => {}
                        };
                    });

                    port_map.remove(&id);
                },

                Message::CSConnectDN(id, buf, port) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                        match value.tx.send(PortMessage::ConnectDN(buf, port)){
                            Ok(_) => {},
                            Err(_) => {}
                        };
                    });
                },

                Message::CSShutdownWrite(id) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                        match value.tx.send(PortMessage::ShutdownWrite){
                            Ok(_) => {},
                            Err(_) => {}
                        };
                    });
                },

                Message::CSData(op, id, buf) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                        match value.tx.send(PortMessage::Data(op, buf)){
                            Ok(_) => {},
                            Err(_) => {}
                        };
                    });
                },
                Message::SCClosePort(id) => {
                    let res = port_map.get(&id).map(|value| {
                        match value.tx.send(PortMessage::ClosePort){
                            Ok(_) => {},
                            Err(_) => {}
                        };

                        try!(stream.write_u8(sc::CLOSE_PORT));
                        try!(stream.write_u32(id));
                        Ok(())
                    });

                    match res {
                        Some(Err(e)) => return Err(e),
                        _ => {}
                    }

                    port_map.remove(&id);
                },

                Message::SCShutdownWrite(id) => {
                    try!(stream.write_u8(sc::SHUTDOWN_WRITE));
                    try!(stream.write_u32(id));
                },

                Message::SCConnectOk(id, buf) => {
                    try!(stream.write_u8(sc::CONNECT_OK));
                    try!(stream.write_u32(id));
                    try!(stream.write_u32(buf.len() as u32));
                    try!(stream.write(&buf[..]));
                },

                Message::SCData(id, buf) => {
                    try!(stream.write_u8(sc::DATA));
                    try!(stream.write_u32(id));
                    try!(stream.write_u32(buf.len() as u32));
                    try!(stream.write(&buf[..]));
                },

                Message::PortDrop(id) => {
                    let remove = if let Some(value)
                        = port_map.get_mut(&id) {
                            value.count = value.count - 1;
                            value.count == 0
                        } else {
                            false
                        };

                    if remove {
                        

                        port_map.remove(&id);
                    }
                },
                Message::CloseTunnel => break
            }
        }
    }

    Ok(())
    
}
