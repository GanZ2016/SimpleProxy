<<<<<<< HEAD
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::channel;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;
use std::vec::Vec;
use std::str::from_utf8;
use std::collections::HashMap;
use std::time::Duration;
use std::net::TcpStream;
use std::net::SocketAddr;
use std::net::lookup_host;
=======
use std::sync::mpsc::{sync_channel, SyncSender, channel, Sender, Receiver};
use std::thread;
use std::vec::Vec;
use std::time::Duration;
use std::collections::HashMap;
use std::net::TcpStream;
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError};
use super::cryptor::Cryptor;
<<<<<<< HEAD
=======
use std::io::{Read,Write};

pub mod cs {
    pub const OPEN_PORT: u8 = 1;
    pub const CLOSE_PORT: u8 = 2;
    pub const SHUTDOWN_WRITE: u8 = 4;
    pub const CONNECT: u8 = 5;
    pub const CONNECT_OK: u8 = 4;
    pub const CONNECT_DOMAIN_NAME: u8 = 6;
    pub const DATA: u8 = 7;
    pub const HEARTBEAT: u8 = 8;
}
pub mod sc {
    pub const CLOSE_PORT: u8 = 1;
    pub const SHUTDOWN_WRITE: u8 = 3;
    pub const CONNECT_OK: u8 = 4;
    pub const DATA: u8 = 5;
    pub const HEARTBEAT_RSP: u8 = 6;
}

>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
// Enumeration Type of Message Transfer in Tunnel
enum Message {
    CSHeartbeat,
    CSOpenPort(u32),
    CSClosePort(u32),
    CSShutdownWrite(u32),
<<<<<<< HEAD
    CSConnectDN(u32, Vec<u8>, u16),
    CSData(u8, u32, Vec<u8>),

    SCClosePort(u32),
    SCShutdownWrite(u32),
    SCConnectOk(u32, Vec<u8>),
    SCData(u32, Vec<u8>),

    PortDrop(u32),
=======
    CSConnectDN(u32, Vec<u8>, u16)
    CSData(u8, u32,Vec<u8>),

    SCClosePort(u32),
    SCShutDownWrite(u32),
    SCConnectOk(u32,Vec<u8>),
    SCData(u32,Vec<u8>),

    TunnelPortDrop(u32),
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
    CloseTunnel,
}
//Enumeration Type for Port Message.
pub enum PortMessage {
<<<<<<< HEAD
    ConnectDN(Vec<u8>),
=======
    ConnectOk(Vec<u8>),
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
    Data(Vec<u8>),
    ShutdownWrite,
    ClosePort,
}

<<<<<<< HEAD
pub struct Tunnel;
=======
pub struct Tunnel{
    tunnel_id:u32,
    core_tx: SyncSender<Message>,
}
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9

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
<<<<<<< HEAD
=======
    host: String,
    port: u16,
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
    count: u32,
    tx: Sender<PortMessage>,
}

type PortMap = HashMap<u32, PortMapValue>;
<<<<<<< HEAD
impl Tunnel {
    pub fn new(stream: TcpStream){
        thread::spawn(move || {
            tunnel_core_task(stream);
        })
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
        let _ = self.tx.send(Message::SCConnectOk(self.id, buf));
    }

    fn write(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::SCData(self.id, buf));
    }

    fn shutdown_write(&self) {
        let _ = self.tx.send(Message::SCShutdownWrite(self.id));
    }

    fn close(&self) {
        let _ = self.tx.send(Message::SCClosePort(self.id));
    }

}
impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::PortDrop(self.port_id));
=======

impl Tunnel {
    pub fn new(stream:TcpStream, key: Vec<u8>) {

        thread::spawn(move || {
            tunnel_core_task(stream, key);
        });
    }
    //  pub fn open_port(&mut self) -> (TunnelWritePort, TunnelReadPort) {
    //     let core_tx1 = self.core_tx.clone();
    //     let core_tx2 = self.core_tx.clone();
    //     let id = self.tunnel_id;
    //     self.tunnel_id += 1;

    //     let (tx, rx) = channel();
    //     let _ = self.core_tx.send(Message::CSOpenPort(id, tx));

    //     (TunnelWritePort { port_id: id, tx: core_tx1 },
    //      TunnelReadPort { port_id: id, tx: core_tx2, rx: rx })
    // }
} 

impl TunnelWritePort {
    pub fn write(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::SCData(self.port_id,buf));
    }

    pub fn connect_ok(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::SCConnectOk(self.port_id, buf));
    }

    // pub fn connect_domain_name(&self, buf: Vec<u8>, port: u16) {
    //     let _ = self.tx.send(Message::CSConnectDN(self.port_id, buf, port));
    // }
    pub fn shutdown_write(&self) {
        let _ = self.tx.send(Message::SCShutDownWrite(self.port_id));
    }

    pub fn close(&self) {
        let _ = self.tx.send(Message::SCClosePort(self.port_id));
    }

}

impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::TunnelPortDrop(self.port_id));
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
    }
}

impl TunnelReadPort {
    pub fn read(&self) -> PortMessage {
<<<<<<< HEAD
        self.rx.recv().unwrap()
    }
}

impl Drop for TunnelReadPort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::PortDrop(self.port_id));
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

fn tunnel_port_read(s: TcpStream, read_port: TunnelReadPort) {
    let mut stream = Tcp::new(s);

    loop {
        match read_port.read() {
            TunnelPortMsg::Data(cs::DATA, buf) => {
                match stream.write(&buf[..]) {
                    Ok(_) => {},
                    Err(_) => {
                        stream.shutdown();
                        break
                    }
                }
            },
            TunnelPortMsg::ShutdownWrite => {
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
                                SocketAddr::V6(addr_v6) =>
                                    TcpStream::connect((addr_v6.ip().clone(), port))
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
            let _ = write!(&mut buf, "{}", addr);
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
    let _ = tunnel_recv_loop(&core_tx, &mut stream);
    stream.shutdown();
    let _ = core_tx.send(Message::CloseTunnel);
}

fn tunnel_recv_loop(core_tx: &SyncSender<Message>,
                    stream: &mut Tcp) -> Result<(), TcpError> {
    let ctr = try!(stream.read_exact(Cryptor::ctr_size()));
    let mut decryptor = Cryptor::with_ctr(ctr);

    let buf = try!(stream.read_exact(VERIFY_DATA.len()));
    let data = decryptor.decrypt(&buf[..]);

    if &data[..] != &VERIFY_DATA[..] {
        return Err(TcpError::ErrorData);
    }
    loop {
        let op = try!(stream.read_u8());
        if op == cs::HEARTBEAT {
            let _ = core_tx.send(Message::CSHeartbeat);
            continue
        }

        let id = try!(stream.read_u32());
        match op {
            cs::OPEN_PORT => {
                let _ = core_tx.send(Message::CSOpenPort(id));
            },
            cs::CLOSE_PORT => {
                let _ = core_tx.send(Message::CSClosePort(id));
            },
            cs::SHUTDOWN_WRITE => {
                let _ = core_tx.send(Message::CSShutdownWrite(id));
            },

            cs::CONNECT_DOMAIN_NAME => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_exact((len - 2) as usize));
                let port = try!(stream.read_u16());
                let _ = core_tx.send(Message::CSConnectDN(id, domain_name, port));
            },

            sc::DATA => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_exact(len as usize));
                let data = decryptor.decrypt(&buf[..]);
                let _ = core_tx.send(Message::SCData(id, data));
            },

            _ => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_exact(len as usize));
                let _ = core_tx.send(Message::CSData(op, id, data));
            }
        }
    }

    Ok(())
}


fn tunnel_core_task(sender: TcpStream){
    let (core_tx, core_rx) = sync_channel(10000);
    let receiver = sender.try_clone().unwrap();
    let core_tx2 = core_tx.clone();

    thread::spawn(move || {
        tunnel_tcp_recv(receiver, core_tx2);
    });

    let mut stream = Tcp::new(sender);
    let mut port_map = PortMap::new();

    let _ = tunnel_loop( &core_tx, &core_rx, &mut stream, &mut port_map);

    stream.shutdown();
    for (_, value) in port_map.iter() {
        let _ = value.tx.send(PortMessage::ClosePort);
    }
}
fn tunnel_loop(tid: u32,
               core_rx: &Receiver<Message>, stream: &mut Tcp,
               port_map: &mut PortMap)
    -> Result<(), TcpError> {


    let timer = Timer::new(HEARTBEAT_INTERVAL_MS);
    let mut alive_time = time::get_time();

    loop {
        select! {
            _ = timer.recv() => {
                let duration = time::get_time() - alive_time;
                if duration.num_milliseconds() > ALIVE_TIMEOUT_TIME_MS {
                    break
                }
                try!(stream.write_u8(cs::HEARTBEAT));
            },

            msg = core_rx.recv() => match msg.unwrap() {
                Message::CSHeartbeat => {
                   alive_time = time::get_time();
                    try!(stream.write_u8(sc::HEARTBEAT_RSP));
                },

            Message::CSOpenPort(id) => {
                    alive_time = time::get_time();
                    let (tx, rx) = channel();
                    port_map.insert(id, PortMapValue { count: 2, tx: tx });

                    let read_port = TunnelReadPort {
                        id: id, tx: core_tx.clone(), rx: rx
                    };
                    let write_port = TunnelWritePort {
                        id: id, tx: core_tx.clone()
                    };

                    thread::spawn(move || {
                        tunnel_port_task(read_port, write_port);
                    });
                },

                Message::CSClosePort(id) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                        let _ = value.tx.send(TunnelPortMsg::ClosePort);
                    });

                    port_map.remove(&id);
                },

                Message::CSConnectDN(id, buf, port) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                        let _ = value.tx.send(TunnelPortMsg::ConnectDN(domain_name, port));
                    });
                },

                Message::CSShutdownWrite(id) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                        let _ = value.tx.send(TunnelPortMsg::ShutdownWrite);
                    });
                },

                Message::CSData(op, id, buf) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                        let _ = value.tx.send(TunnelPortMsg::Data(op, buf));
                    });
                },
                Message::SCClosePort(id) => {
                    match port_map.get(&id) {
                        Some(value) => {
                            info!("{}.{}: server close {}:{}",
                                  tid, id, value.host, value.port);
                        },
                        None => {
                            info!("{}.{}: server close unknown client",
                                  tid, id);
                        }
                    }

                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                        let _ = value.tx.send(PortMessage::ClosePort);
                    });

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

=======
        match self.rx.recv() {
            ok(msg) => msg,
            Err(_) => PortMessage::ClosePort
        }
    }
}
impl Drop for TunnelReadPort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::TunnelPortDrop(self.port_id));
    }
}
fn tunnel_tcp_recv(key: Vec<u8>, receiver: TcpStream,
                   core_tx: SyncSender<Message>) {
    let mut stream = Tcp::new(receiver);
    let _ = tunnel_recv_loop(&key, &core_tx, &mut stream);
    stream.shutdown();
    let _ = core_tx.send(Tunnel::CloseTunnel);

}

fn tunnel_recv_loop(key: &Vec<u8>, stream:TcpStream, core_tx: &SyncSender<Message>) -> Result<(),TcpError> {
    loop {
    let op = try!(stream.read_u8());

    if op == HEARTBEAT {
        let _ = core_tx.send(Message::CSHeartbeat);
        continue
    }
    let port_id = try!(stream.read);
    match op {
        cs::CLOSE_PORT => {
            let _ = core_tx.send(Message::CSClosePort(port_id));
        },

        cs::SHUTDOWN_WRITE => {
            let _ = core_tx.send(Message::CSShutdownWrite(id));
        },

        cs::OPEN_PORT => {
            let _ = core_tx.send(Message::CSOpenPort(id));
        },

        cs::CONNECT_DOMAIN_NAME = > {
            let len = try!(stream.read_u32());
            let buf = try!(stream.read_exact(len-2 as usize));
            let port = try!(stream.read_16());
            let data = decryptor.decrypt(&buf[..]);
            let _ = core_tx.send(Message::CSConnectDN(id, data, port));
        }

        _ => {
            let len = try!(stream.read_u32());
            let buf = try!(stream.read_exact(len as usize));
            let data = decryptor.decrypt(&buf[..]);
            let _ = core_tx.send(Message::CSData(op, id, data));
        },

            _ => break
        }
    }
}
>>>>>>> d2ed3b03611c67fe008ead5868c3a1195deb1de9
