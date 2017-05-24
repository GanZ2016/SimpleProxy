use std::sync::mpsc::{sync_channel, SyncSender, channel, Sender, Receiver};
use std::thread;
use std::vec::Vec;
use std::time::Duration;
use std::collections::HashMap;
use std::net::TcpStream;
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError};
use super::cryptor::Cryptor;
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

// Enumeration Type of Message Transfer in Tunnel
enum Message {
    CSHeartbeat,
    CSOpenPort(u32),
    CSClosePort(u32),
    CSShutdownWrite(u32),
    CSConnectDN(u32, Vec<u8>, u16)
    CSData(u8, u32,Vec<u8>),

    SCClosePort(u32),
    SCShutDownWrite(u32),
    SCConnectOk(u32,Vec<u8>),
    SCData(u32,Vec<u8>),

    TunnelPortDrop(u32),
    CloseTunnel,
}
//Enumeration Type for Port Message.
pub enum PortMessage {
    ConnectOk(Vec<u8>),
    Data(Vec<u8>),
    ShutdownWrite,
    ClosePort,
}

pub struct Tunnel{
    tunnel_id:u32,
    core_tx: SyncSender<Message>,
}

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
    host: String,
    port: u16,
    count: u32,
    tx: Sender<PortMessage>,
}

type PortMap = HashMap<u32, PortMapValue>;

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
    }
}

impl TunnelReadPort {
    pub fn read(&self) -> PortMessage {
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