use std::sync::mpsc::{sync_channel, SyncSender, channel, Sender, Receiver};
use std::thread;
use std::vec::Vec;
use std::time::Duration;
use std::collections::HashMap;
use std::net::TcpStream;
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError};
use super::protocol::{
    VERIFY_DATA, cs, sc,
    HEARTBEAT_INTERVAL_MS,
    ALIVE_TIMEOUT_TIME_MS
};
// Enumeration Type of Message Transfer in Tunnel

enum Message {
    CSOpenPort(u32, Sender<PortMessage>),
    CSConnect(u32,Vec<u8>),
    CSShutdownWrite(u32),
    CSConnectDN(u32, Vec<u8>, u16),
    CSClosePort(u32),
    CSData(u32,Vec<u8>),

    SCHeartbeat,
    SCClosePort(u32),
    SCShutdownWrite(u32),
    SCConnectOk(u32,Vec<u8>),
    SCData(u32,Vec<u8>),

    PortDrop(u32)
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
    pub fn new(tid:u32, server_addr: String) ->Tunnel {
        let (tx,rx) = sync_channel(10000);
        let tx2 = tx.clone();

        thread::spawn(move || {
            tunnel_core_task(tid,server_addr, rx,tx);
        });
        Tunnel {tunnel_id:1,core_tx:tx2}
    }
     pub fn open_port(&mut self) -> (TunnelWritePort, TunnelReadPort) {
        let core_tx1 = self.core_tx.clone();
        let core_tx2 = self.core_tx.clone();
        let id = self.tunnel_id;
        self.tunnel_id += 1;

        let (tx, rx) = channel();
        self.core_tx.send(Message::CSOpenPort(id, tx)).unwrap();

        (TunnelWritePort { port_id: id, tx: core_tx1 },
         TunnelReadPort { port_id: id, tx: core_tx2, rx: rx })
    }
} 

impl TunnelWritePort {
    pub fn write(&self, buf: Vec<u8>) {
        self.tx.send(Message::CSData(self.port_id,buf)).unwrap();
    }

    pub fn connect(&self, buf: Vec<u8>) {
        let _ = self.tx.send(Message::CSConnect(self.port_id, buf)).unwrap();
    }

    pub fn connect_domain_name(&self, buf: Vec<u8>, port: u16) {
        let _ = self.tx.send(Message::CSConnectDN(self.port_id, buf, port)).unwrap();
    }
    pub fn shutdown_write(&self) {
        let _ = self.tx.send(Message::CSShutdownWrite(self.port_id)).unwrap();
    }

    pub fn close(&self) {
        let _ = self.tx.send(Message::CSClosePort(self.port_id)).unwrap();
    }

}
impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::PortDrop(self.port_id)).unwrap();
    }
}

impl TunnelReadPort {
    pub fn read(&self) -> PortMessage {
        match self.rx.recv() {
            Ok(msg) => msg,
            Err(_) => PortMessage::ClosePort
        }
    }
}

impl Drop for TunnelReadPort {
    fn drop(&mut self) {
        let _ = self.tx.send(Message::PortDrop(self.port_id)).unwrap();
    }
}
fn tunnel_tcp_recv( receiver: TcpStream,
                   core_tx: SyncSender<Message>) {
    let mut stream = Tcp::new(receiver);
    match tunnel_recv_loop( &core_tx, &mut stream) {
        _ => Ok(_),
        Err(e) => TcpError(e),
    } 
    stream.shutdown();
}

fn tunnel_recv_loop(core_tx: &SyncSender<Message>,
                    stream: &mut Tcp) -> Result<(), TcpError> {

    loop {
        let op = try!(stream.read_u8());
        if op == sc::HEARTBEAT_RSP {
            core_tx.send(Message::SCHeartbeat).unwrap();
            continue
        }

        let id = try!(stream.read_u32());
        match op {
            sc::CLOSE_PORT => {
                let _ = core_tx.send(Message::SCClosePort(id)).unwrap();
            },

            sc::SHUTDOWN_WRITE => {
                let _ = core_tx.send(Message::SCShutdownWrite(id)).unwrap();
            },

            sc::CONNECT_OK => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_exact(len as usize));
                let _ = core_tx.send(Message::SCConnectOk(id, buf)).unwrap();
            },

            sc::DATA => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_exact(len as usize));
                let _ = core_tx.send(Message::SCData(id, buf)).unwrap();
            },

            _ => break
        }
    }

    Ok(())
}


fn tunnel_core_task(tid: u32, server_addr: String,
                    core_rx: Receiver<Message>,
                    core_tx: SyncSender<Message>){
    let sender = match TcpStream::connect(&server_addr[..]){
        Ok(sender) => sender,
        Err(_) => {
            thread::sleep(Duration::from_millis(1000));
            thread::spawn(move || {
                tunnel_core_task(tid, server_addr, core_rx, core_tx);
            });
            return
        }
    };
    
    let receiver = sender.try_clone().unwrap();
    let core_tx2 = core_tx.clone();

    thread::spawn(move || {
        tunnel_tcp_recv(receiver, core_tx2)
    });

    let mut stream = Tcp::new(sender);
    let mut port_map = PortMap::new();

    match tunnel_loop(tid, &core_rx, &mut stream, &mut port_map){
        Ok() => {},
        Err(_) => {

        }
    };
    info!("tunnel {} broken", tid);

    stream.shutdown();
    for (_, value) in port_map.iter() {
        let _ = value.tx.send(PortMessage::ClosePort);
    }

    thread::spawn(move || {
        tunnel_core_task(tid, server_addr, core_rx, core_tx);
    });
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
                Message::CSOpenPort(id, tx) => {
                    port_map.insert(id, PortMapValue {
                        count: 2, tx: tx, host: String::new(), port: 0 });

                    try!(stream.write_u8(cs::OPEN_PORT));
                    try!(stream.write_u32(id));
                },

                Message::CSConnect(id, buf) => {

                    try!(stream.write_u8(cs::CONNECT));
                    try!(stream.write_u32(id));
                    try!(stream.write_u32(buf.len() as u32));
                    try!(stream.write(&buf[..]));
                },

                Message::CSConnectDN(id, buf, port) => {
                    let host = String::from_utf8(buf.clone()).
                        unwrap_or(String::new());

                    if let Some(value) = port_map.get_mut(&id) {
                        value.host = host.clone();
                        value.port = port;
                    }

                    info!("{}.{}: connecting {}:{}", tid, id, host, port);


                    try!(stream.write_u8(cs::CONNECT_DOMAIN_NAME));
                    try!(stream.write_u32(id));
                    try!(stream.write_u32(buf.len() as u32 + 2));
                    try!(stream.write(&buf[..]));
                    try!(stream.write_u16(port));
                },

                Message::CSShutdownWrite(id) => {
                    match port_map.get(&id) {
                        Some(value) => {
                            info!("{}.{}: client shutdown write {}:{}",
                                  tid, id, value.host, value.port);
                        },
                        None => {
                            info!("{}.{}: client shutdown write unknown server",
                                  tid, id);
                        }
                    }

                    try!(stream.write_u8(cs::SHUTDOWN_WRITE));
                    try!(stream.write_u32(id));
                },

                Message::CSData(id, buf) => {

                    try!(stream.write_u8(cs::DATA));
                    try!(stream.write_u32(id));
                    try!(stream.write_u32(buf.len() as u32));
                    try!(stream.write(&buf[..]));
                },

                Message::CSClosePort(id) => {
                    match port_map.get(&id) {
                        Some(value) => {
                            info!("{}.{}: client close {}:{}",
                                  tid, id, value.host, value.port);
                        },
                        None => {
                            info!("{}.{}: client close unknown server",
                                  tid, id);
                        }
                    }

                    let res = port_map.get(&id).map(|value| {
                    value.tx.send(PortMessage::ClosePort).unwrap();

                    try!(stream.write_u8(cs::CLOSE_PORT));
                    try!(stream.write_u32(id));
                    Ok(())
                    });

                    match res {
                        Some(Err(e)) => return Err(e),
                        _ => {}
                    }

                    port_map.remove(&id);
                },

                Message::SCHeartbeat => {
                    alive_time = time::get_time();
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
                    value.tx.send(PortMessage::ClosePort).unwrap();
                    });

                    port_map.remove(&id);
                },

                Message::SCShutdownWrite(id) => {
                    match port_map.get(&id) {
                        Some(value) => {
                            info!("{}.{}: server shutdown write {}:{}",
                                  tid, id, value.host, value.port);
                        },
                        None => {
                            info!("{}.{}: server shutdown write unknown client",
                                  tid, id);
                        }
                    }

                    alive_time = time::get_time();
                    port_map.get(&id).map(|value| {
                    value.tx.send(PortMessage::ShutdownWrite).unwrap();
                    });
                },

                Message::SCConnectOk(id, buf) => {
                    match port_map.get(&id) {
                        Some(value) => {
                            info!("{}.{}: connect {}:{} ok",
                                  tid, id, value.host, value.port);
                        },
                        None => {
                            info!("{}.{}: connect unknown server ok",
                                  tid, id);
                        }
                    }

                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                    value.tx.send(PortMessage::ConnectOk(buf)).unwrap();
                    });
                },

                Message::SCData(id, buf) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                    value.tx.send(PortMessage::Data(buf)).unwrap();
                    });
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
                        match port_map.get(&id) {
                            Some(value) => {
                                info!("{}.{}: drop tunnel port {}:{}",
                                      tid, id, value.host, value.port);
                            },
                            None => {
                                info!("{}.{}: drop unknown tunnel port",
                                      tid, id);
                            }
                        }

                        port_map.remove(&id);
                    }
                }
            }
        }
    }

    Ok(())
}

