use std::sync::mpsc::{sync_channel, SyncSender, channel, Sender, Receiver};
use std::thread;
use std::vec::Vec;
use std::time::Duration;
use std::collections::HashMap;
use std::net::{TcpStream,ToSocketAddrs};
use std::io::Write;
use std::str::from_utf8;
use time;
use super::timer::Timer;
use super::socks5::{Tcp,TcpError,success_reply,ConnectInfo,connect_target};
use super::protocol::{
    cs, sc,
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
    /// Tunnel is the bridge connects client and server.
impl Tunnel {
    /// Returns a tunnel which has one sync_channel 
    /// Based on the SSH remote port forwarding
    ///
    /// # Arguments
    /// 
    /// * `tid` - A unsigned integer of 32bit that shows the index of tunnel
    /// * `server_add`` - A IPv4 address including port that server is listening.
    ///
    /// # Example
    ///
    /// ```
    /// use doc::Tunnel;
    /// let new_tunnel = Tunnel::new(0, "127.0.0.1:1080");
    /// ```
    
    pub fn new(tid:u32, server_addr: String) ->Tunnel {
        let (tx,rx) = sync_channel(10000);
        let tx2 = tx.clone();

        thread::spawn(move || {
            tunnel_core_task(tid,server_addr, rx,tx);
        });
        Tunnel {tunnel_id:1,core_tx:tx2}
    }
    /// For each web request open a corresponding port
    ///
    /// # Example
    ///
    /// ```
    /// use doc::Tunnel;
    /// let new_tunnel = Tunnel::new(0, "127.0.0.1:1080");
    /// let (write_port, read_port) = new_tunnel.open_port();
    /// ```
     pub fn open_port(&mut self) -> (TunnelWritePort, TunnelReadPort) {
        let core_tx1 = self.core_tx.clone();
        let core_tx2 = self.core_tx.clone();
        let id = self.tunnel_id;
        self.tunnel_id += 1;

        let (tx, rx) = channel();
        match self.core_tx.send(Message::CSOpenPort(id, tx)){
            Ok(_) => {},
            Err(_) => {}
        };

        (TunnelWritePort { port_id: id, tx: core_tx1 },
         TunnelReadPort { port_id: id, tx: core_tx2, rx: rx })
    }
} 
///  Extract message from port and send it to sync_channel.
impl TunnelWritePort {
    /// Writing data for a specific port to tunnel
    ///
    /// # Arguments
    ///
    /// *`buf` - A vector of u8 that is the data sending to tunnel
    ///
    /// # Example
    ///
    /// ```
    /// use doc::TunnelWritePort;
    /// let (tx, rx) = channel();
    /// let write_port = TunnelWritePort { port_id: 0, tx: tx }
    /// let mut buf = Vec::new();
    /// write_port.write(buf.clone());
    /// assert_eq!(rx.recv().unwrap(), Message::CSData(0,buf));
    /// ```

    pub fn write(&self, buf: Vec<u8>) {
        match self.tx.send(Message::CSData(self.port_id,buf)){
            Ok(_) =>{},
            Err(_) =>{}
        };
    }
    /// Connect port in the tunnel
    /// # Example
    ///
    /// ```
    /// use doc::TunnelWritePort;
    /// let (tx, rx) = channel();
    /// let write_port = TunnelWritePort { port_id: 0, tx: tx }
    /// let mut buf = Vec::new();
    /// write_port.connect(buf.clone());
    /// assert_eq!(rx.recv().unwrap(), Message::CSConnect(0,buf));
    /// ```
    pub fn connect(&self, buf: Vec<u8>) {
        match self.tx.send(Message::CSConnect(self.port_id, buf)){
            Ok(_) =>{},
            Err(_) => {}
        };
    }

    /// Connect domain name of a specific port through tunnel
    /// # Example
    ///
    /// ```
    /// use doc::TunnelWritePort;
    /// let (tx, rx) = channel();
    /// let write_port = TunnelWritePort { port_id: 0, tx: tx }
    /// let mut buf = Vec::new();
    /// write_port.connect_domain_name(buf.clone(),2020);
    /// assert_eq!(rx.recv().unwrap(), Message::CSConnectDN(0,buf,2020));
    /// ```
    pub fn connect_domain_name(&self, buf: Vec<u8>, port: u16) {
        match self.tx.send(Message::CSConnectDN(self.port_id, buf, port)){
            Ok(_) => {},
            Err(_) => {}
        }
    }
    /// shutdown the writing of port.
    /// # Example
    ///
    /// ```
    /// use doc::TunnelWritePort;
    /// let (tx, rx) = channel();
    /// let write_port = TunnelWritePort { port_id: 0, tx: tx }
    /// let mut buf = Vec::new();
    /// write_port.connect_domain_name(0);
    /// assert_eq!(rx.recv().unwrap(), Message::CSConnectDN(0));
    /// ```
    pub fn shutdown_write(&self) {
        match self.tx.send(Message::CSShutdownWrite(self.port_id)){
            Ok(_) => {},
            Err(_) => {} 
        }
    }
    /// close port.
    /// # Example
    ///
    /// ```
    /// use doc::TunnelWritePort;
    /// let (tx, rx) = channel();
    /// let write_port = TunnelWritePort { port_id: 0, tx: tx }
    /// let mut buf = Vec::new();
    /// write_port.close(0);
    /// assert_eq!(rx.recv().unwrap(), Message::CSClosePort(0));
    /// ```
    pub fn close(&self) {
        match self.tx.send(Message::CSClosePort(self.port_id)){
            Ok(_) => {},
            Err(_) => {}
        }
    }

}
/// drop port
impl Drop for TunnelWritePort {
    fn drop(&mut self) {
        match self.tx.send(Message::PortDrop(self.port_id)){
            Ok(_) =>{},
            Err(_) => {}
        };
    }
}
    /// TunnelReadPort got the message from TCP stream and send it to port Message handler
impl TunnelReadPort {
    /// Directly read portmessage.
    ///
    pub fn read(&self) -> PortMessage {
        match self.rx.recv() {
            Ok(msg) => msg,
            Err(_) => PortMessage::ClosePort
        }
    }
}
///the message directly send to tunnel message to drop port.
impl Drop for TunnelReadPort {
    fn drop(&mut self) {
        match self.tx.send(Message::PortDrop(self.port_id)){
            Ok(_) =>{},
            Err(_) =>{}
        };
    }
}
    // Read Port meesage and check address, and get reply from server
    // if is "DATA", write data into buffer and write into tcpstream
    // if is "SHUTDOWN_WRITE", stop write to stream
    // eles, shutdown the connection
pub fn tunnel_read_port(tcpstream: TcpStream, port: TunnelReadPort) {
    let addr = match port.read() {
        PortMessage::ConnectOk(buf) =>{
            from_utf8(&buf[..]).unwrap().to_socket_addrs().unwrap().nth(0)
        },
        _ => None,
    };
    let mut stream = Tcp::new(tcpstream.try_clone().unwrap());
    let addr_ok = match addr {
        Some(addr) => success_reply(&mut stream,addr).is_ok(),
        None => false,
    };
    if !addr_ok{ stream.shutdown();}
    while addr_ok {
        let buf = match port.read() {
            PortMessage::Data(buf) => buf,
            PortMessage::ShutdownWrite =>{
                stream.shutdown_write();
                break
            },
            _ => {
                stream.shutdown();
                break
            },
        };

        match stream.write(&buf[..]) {
            Ok(_) => {},
            Err(_) => {
                stream.shutdown();
                break
            },
        }
    }
}

pub fn tunnel_write_port(tcpstream: TcpStream, write_port: TunnelWritePort, read_port:TunnelReadPort) {
    let mut stream = Tcp::new(tcpstream.try_clone().unwrap());
    // get connection info: address or domain + port
    match connect_target(&mut stream) {
        Ok(ConnectInfo::Addr(addr)) => {
            let mut buf = Vec::new();
            let _ =write!(&mut buf, "{}", addr);
            write_port.connect(buf);
        },
       Ok(ConnectInfo::Domain(domain_name,port)) => {
           write_port.connect_domain_name(domain_name,port);
       },
       _ =>{
           return write_port.close();
       }
    };
    // create thread
    // check address, and get reply from server
    // if is "DATA", write data into buffer and write into tcpstream
    // if is "SHUTDOWN_WRITE", stop write to stream
    // eles, shutdown the connection
    thread::spawn(move || {
        tunnel_read_port(tcpstream,read_port);
    });
    //Write data through write_port until EoF or error
    loop {
        match stream.read_at_most(1024) {
            Ok(buf) => {
                write_port.write(buf);
            },
            Err(TcpError::Eof) => {
                stream.shutdown_read(); //stop read from stream
                write_port.shutdown_write(); //stop write to port
                break
            },
            Err(_) => {
                //shutdown stream and close the port
                stream.shutdown();
                write_port.close();
                break
            }
        }
    }
}
/// Tcp listener for client. Extracting the header and process data.
fn tunnel_tcp_recv( receiver: TcpStream,
                   core_tx: SyncSender<Message>) {
    let mut stream = Tcp::new(receiver);
    match tunnel_recv_loop( &core_tx, &mut stream){
        Ok(_) => {},
        Err(_) => {}
    };
    stream.shutdown();
}
/// The loop to keep listening TCP stream (Tunnel), extracting the message of tunnel
/// and send it to the sync_channel receiver which is the Tunnel message handler.
fn tunnel_recv_loop(core_tx: &SyncSender<Message>,
                    stream: &mut Tcp) -> Result<(), TcpError> {

    loop {
        let op = try!(stream.read_u8());
        // HEARTBEAT is the initialization of communication
        if op == sc::HEARTBEAT_RSP {
            match core_tx.send(Message::SCHeartbeat){
                Ok(_) => {},
                Err(_) => {}
            };
            continue
        }

        let id = try!(stream.read_u32());
        match op {
            sc::CLOSE_PORT => {
                match core_tx.send(Message::SCClosePort(id)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            sc::SHUTDOWN_WRITE => {
                match core_tx.send(Message::SCShutdownWrite(id)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            sc::CONNECT_OK => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_size(len as usize));
                match core_tx.send(Message::SCConnectOk(id, buf)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            sc::DATA => {
                let len = try!(stream.read_u32());
                let buf = try!(stream.read_size(len as usize));
                match core_tx.send(Message::SCData(id, buf)){
                    Ok(_) => {},
                    Err(_) => {}
                };
            },

            _ => break
        }
    }

    Ok(())
}
//Main function for Tunnel. First create an TCP stream which is the connection between client and server.
// Then use message handler which is tunnel_loop to handle message between client and server.

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
        Ok(_) => {},
        Err(_) => {}
    };
    info!("tunnel {} broken", tid);

    stream.shutdown();
    for (_, value) in port_map.iter() {
        match value.tx.send(PortMessage::ClosePort){
            Ok(_) => {},
            Err(_) => {}
        }
    }

    thread::spawn(move || {
        tunnel_core_task(tid, server_addr, core_rx, core_tx);
    });
}
//
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
                    match value.tx.send(PortMessage::ClosePort){
                        Ok(_) => {},
                        Err(_) => {}
                    };

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
                    match value.tx.send(PortMessage::ClosePort){
                        Ok(_) => {},
                        Err(_) => {}
                    };
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
                    match value.tx.send(PortMessage::ShutdownWrite){
                        Ok(_) => {},
                        Err(_) => {}
                    };
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
                    match value.tx.send(PortMessage::ConnectOk(buf)){
                        Ok(_) => {},
                        Err(_) => {}
                    };
                    });
                },

                Message::SCData(id, buf) => {
                    alive_time = time::get_time();
                    port_map.get(&id).map(move |value| {
                    match value.tx.send(PortMessage::Data(buf)){
                        Ok(_) => {},
                        Err(_) => {}
                    };
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

