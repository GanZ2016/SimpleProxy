use std::sync::mpsc::sync_channel;
use std::sync::mpsc::channel;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::thread;
use std::collections::HashMap;
use std::net::TcpStream;
use std::vec::Vec;
use std::time::Duration;
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError};

enum Message {
    CSOpenport(u32, Sender<PortMessage>),
    CSConnect(u32,Vec<u8>),
    CSShutdownWrite(u32),
    CSConnectDN(u32, Vec<u8>, u16)
    CSClosePort(u32),
    CSData(u32,Vec<u8>),

    SCHeartbeat,
    SCClosePort(u32),
    SCShutDownWrite(u32),
    SCConnectOk(u32,Vec<u8>),
    SCData(u32,Vec<u8>),

    PortDrop(u32)
}
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
impl Tunnel {
    pub fn new(tid:u32, server_addr: String, key: Vec<u8>) ->Tunnel {
        let (tx,rx) = sync_channel(10000);
        let tx2 = tx.clone();

        thread::spawn(move || {
            tunnel_core_task(tid,server_addr,key,rx,tx);
        })
        Tunnel {tunnel_id:1,core_tx:tx2}
    }
     pub fn open_port(&mut self) -> (TunnelWritePort, TunnelReadPort) {
        let core_tx1 = self.core_tx.clone();
        let core_tx2 = self.core_tx.clone();
        let id = self.tunnel_id;
        self.tunnel_id += 1;

        let (tx, rx) = channel();
        let _ = self.core_tx.send(Message::CSOpenPort(id, tx));

        (TunnelWritePort { port_id: id, tx: core_tx1 },
         TunnelReadPort { port_id: id, tx: core_tx2, rx: rx })
    }
} 

impl TunnelWritePort {
    pub fn write(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::CSData(self.port_id,buf));
    }

    pub fn connect(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::CSConnect(self.port.id, buf));
    }

    pub fn connect_domain_name(&self, buf: Vec<u8>, port: u16) {
        let _ = self.tx.send(Message::CSConnectDN(self.port_id, buf, port));
    }


}
impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::PortDrop(self.port_id));
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
        let _ = self.tx.send(Message::PortDrop(self.port_id));
    }
}