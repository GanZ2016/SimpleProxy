//reference 
//http://wsfdl.com/python/2016/08/19/SS5_protocol.html

use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, Shutdown,TcpStream};
use std::io::{Read,Write,Error};
use std::vec::Vec;
const SOCK_V5:u8 = 5;
const RSV: u8 = 0;
const ATYP_IP_V4: u8 = 1;
const ATYP_DOMAINNAME: u8 = 3;
const CMD_CONNECT: u8 = 1;
const METHOD_NO_AUTH: u8 = 0;
const METHOD_NO_ACCP: u8 = 0xFF;

pub struct Tcp {
    stream : TcpStream
}

#[derive(Debug)]
pub enum TcpError {
    Eof,
    IoError(Error),
    ErrorData,
}

pub enum ConnectInfo {
    Addr(SocketAddr),
    Domain(Vec<u8>, u16),
    Unknown,
}


// We build a struct Tcp for TcpStream and "read", "write" functions for Tcp.

impl Tcp {
    //  Tcp::new -> new TcpStream
    pub fn new(stream: TcpStream) -> Tcp {
        Tcp { stream: stream }
    }

    // Tcp::read_buf -> pull some bytes from Tcp into the specified buffer(vector of u8)
    pub fn read_buf(&mut self, buf: &mut [u8]) ->Result<(),TcpError>{
        let mut l = 0;
        while l < buf.len() {
            match self.stream.read(&mut buf[l..]) {
                Ok(0) => return Err(TcpError::Eof),
                Ok(res) => l += res,
                Err(e) => return Err(TcpError::IoError(e)),
            }
        }
        return Ok(());
    }

    // Tcp::read_size -> read some bits while return a vector of u8 with particular size
    pub fn read_size(&mut self, size: usize) -> Result<Vec<u8>, TcpError>{
        let mut buf = Vec::with_capacity(size);
        unsafe {
            buf.set_len(size);
            }
        match self.read_buf(&mut buf) {
            Ok(_) => return Ok(buf),
            Err(e) => return Err(e),
        }
    }

    pub fn read_at_most(&mut self, size: usize) -> Result<Vec<u8>, TcpError> {
        let mut buf = Vec::with_capacity(size);
        unsafe {
            buf.set_len(size);
            }
        match self.stream.read(&mut buf) {
            Ok(0) => return Err(TcpError::Eof),
            Ok(res) => unsafe {
            buf.set_len(res);
            },
            Err(e) => return Err(TcpError::IoError(e)),
        }
        Ok(buf)
    }



    // Tcp::read_u8 -> read at most 8 bits 
    pub fn read_u8(&mut self) -> Result<u8, TcpError> {
        let mut buf = [0u8];
        try!(self.read_buf(&mut buf));
        return Ok(buf[0]);
    }

    // // Tcp::set_16 u8 -> vector to u16
    // //http://stackoverflow.com/questions/33968870/temporarily-transmute-u8-to-u16
    // pub fn set_u16(a: &mut [u8], mut v: u16) {
    //     a[0] = v as u8;
    //     a[1] = (v >> 8) as u8;
    // }
    // Tcp::read_u16 -> read at most 16 bits
    pub fn read_u16(&mut self) -> Result<u16, TcpError> {
        let mut buf = [0u8; 2];
        try!(self.read_buf(&mut buf));
        let res = unsafe { *(buf.as_ptr() as *const u16)};
        return Ok(u16::from_be(res));
    }
    // pub fn set_u32(a: &mut [u8], mut v: u32) {
    //     a[0] = v as u8;
    //     a[1] = (v >> 8) as u8;
    //     a[2] = (v >> 8) as u8;
    //     a[3] = (v >> 8) as u8;
    // }
    pub fn read_u32(&mut self) -> Result<u32, TcpError> {
        let mut buf = [0u8; 4];
        try!(self.read_buf(&mut buf));
        let res = unsafe { *(buf.as_ptr() as *const u32)};
        return Ok(u32::from_be(res));
    }

    pub fn shutdown(&mut self) {
        self.stream.shutdown(Shutdown::Both);
    }

    pub fn shutdown_write(&mut self) {
        self.stream.shutdown(Shutdown::Write);
    }

    pub fn shutdown_read(&mut self) {
        self.stream.shutdown(Shutdown::Read);
    }


    // Tcp::write -> write a buffer(vector of u8) into Tcp
    pub fn write(&mut self, buf:&[u8]) ->Result<(),TcpError>{
        let mut l = 0;
        while l < buf.len() {
            match self.stream.write(&buf[l..]) {
                //Ok(0) => return Err(TcpError::ErrorData),
                Ok(res) => l += res,
                Err(e) => return Err(TcpError::IoError(e)),
            }
        }
        return Ok(());
    }
    // Tcp::write_u8 -> write at most 8 bits
    pub fn write_u8(&mut self, v: u8) -> Result<(),TcpError> {
        let buf = [v];
        self.write(&buf)
    }
    //Tcp::write_u16 -> write at most 16 bits
    pub fn write_u16(&mut self, v: u16) -> Result<(),TcpError> {
        let mut buf = [0u8;2];
        // let mut res = 0 as u16;
        unsafe { *(buf.as_ptr() as *mut u16) = v.to_be(); }
        self.write(&buf)
    }
    pub fn write_u32(&mut self, v: u32) -> Result<(),TcpError> {
        let mut buf = [0u8;4];
        unsafe { *(buf.as_ptr() as *mut u32) = v.to_be(); }
        self.write(&buf)
    }

}

// Tcp::get_method ->  get the connection method from clients message
// +----+----------+----------+
// |VER | NMETHODS | METHODS  |
// +----+----------+----------+
// | 1  |    1     | 1 to 255 |
// +----+----------+----------+ 

// VER:      set to X'05' for this version of the protocol
// NMETHODS: the number of method identifier octets that appear in the METHODS field
// METHODS:  the values currently defined for METHOD are
//        o  X'00' NO AUTHENTICATION REQUIRED
//        o  X'01' GSSAPI
//        o  X'02' USERNAME/PASSWORD
//        o  X'03' to X'7F' IANA ASSIGNED
//        o  X'80' to X'FE' RESERVED FOR PRIVATE METHODS
//        o  X'FF' NO ACCEPTABLE METHODS
//
// Since we're going to make an no authentication server, the message should be:
//  '\x05\x01\x00'
// When receiving this message, the server will choose a method that supports both sides,
// and return the message
// +----+--------+
// |VER | METHOD |
// +----+--------+
// | 5  |   0    |
// +----+--------+ 
pub fn get_method(stream: &mut Tcp) -> Result<u8,TcpError> {
    
    match stream.read_u8() {
        Ok(ver) => {
            if ver == SOCK_V5 {
                match stream.read_u8() {
                    Ok(nmethod) => {
                        if nmethod == 1 {
                            match stream.read_u8(){
                                Ok(methods) =>{
                                    if methods == METHOD_NO_AUTH {
                                        return Ok(METHOD_NO_AUTH);
                                    }
                                    else {
                                        return Ok(METHOD_NO_ACCP);
                                    }
                                },
                                Err(e) => return Err(e),
                            };
                        }
                        else{panic!("WRONG NMETHOD");}
                    },
                    Err(e) => return Err(e),
                }
            }
            else {
                return Ok(METHOD_NO_AUTH);
            }
        },
        Err(e) => return Err(e),
    }
}
// The client send message to the server, including the address and port of the external server.
// +----+-----+-------+------+----------+----------+
// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

// VER:  protocol version, X'05'
// CMD:
//       o  CONNECT X'01'
//       o  BIND X'02'
//       o  UDP ASSOCIATE X'03'
// RSV:  RESERVED, X'00'
// ATYP: address type of following address
//       o  IP V4 address: X'01'
//       o  DOMAINNAME: X'03'
//       o  IP V6 address: X'04'
// DST.ADDR:  desired destination address
// DST.PORT:  desired destination port in network octet order

// In out system the message should be 
// ATYP: IPV4 '\x05\x01\x00\x03\x0b8.8.8.8\x01\xbb'
// ATYP: DOMAINNAME '\x05\x01\x00\x03\x0bxxx.com\x01\xbb'
// We check the message and return ConnectInfo

pub fn connect_target(stream: &mut Tcp) -> Result<ConnectInfo,TcpError> {
    let method = try!(get_method(stream)); // get method(0)
    try!(stream.write(&[SOCK_V5, method])); 
    let mut recv = [0u8;4];
    try!(stream.read_buf(&mut recv));
    if recv[0] != SOCK_V5 || recv[2] != RSV {
        return Err(TcpError::ErrorData);
    }
    // Only connect cmd is supported.
    if recv[1] != CMD_CONNECT {
        return Ok(ConnectInfo::Unknown);
    } 
    let addr_type = recv[3];
    if addr_type == ATYP_IP_V4 {
        // GET IPV4 address    
        let mut addr_ipv4 = [0u8;4];
        try!(stream.read_buf(&mut addr_ipv4));
        // GET port
        //let mut port_buf = [0u8];
        //try!(stream.read_buf(&mut port));
        let port = try!(stream.read_u16());
        return Ok(ConnectInfo::Addr(
            SocketAddr::V4(
                SocketAddrV4::new(
                    Ipv4Addr::new(
                        addr_ipv4[3],addr_ipv4[2],addr_ipv4[1],addr_ipv4[0]),port)
        )));

    }
    else {
        if addr_type == ATYP_DOMAINNAME {
            //GET domain name
            let len = try!(stream.read_u8());
            let domain = try!(stream.read_size(len as usize));
            let port = try!(stream.read_u16());
            return Ok(ConnectInfo::Domain(domain,port));
        }
        else
        {
            //only ipv4 addr or domain name is supported.
            return Ok(ConnectInfo::Unknown);
        }
    }
}

//  TODO: The SOCK server evaluates the request and establishes 
//  a TCP link with the external server
// +----+-----+-------+------+----------+----------+
// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

// VER:   protocol version, X'05'
// REP:   Reply field:
//        o  X'00' succeeded
//        o  X'01' general SOCKS server failure
//        o  X'02' connection not allowed by ruleset
//        o  X'03' Network unreachable
//        o  X'04' Host unreachable
//        o  X'05' Connection refused
//        o  X'06' TTL expired
//        o  X'07' Command not supported
//        o  X'08' Address type not supported
//        o  X'09' to X'FF' unassigned
// RSV:   RESERVED, X'00'
// ATYP:  address type of following address
//        o  IP V4 address: X'01'
//        o  DOMAINNAME: X'03'
//        o  IP V6 address: X'04'
// BND.ADDR:   server bound address
// BND.PORT:   server bound port in network octet order

// if succefully connected, it should return below message to clint:
// "\x05\x00\x00\x01\BND.ADDR\BND.PORT:\"
pub fn get_reply(stream: &mut Tcp, addr: SocketAddr, rep_info:u8) -> Result<(),TcpError>{
    let buf = [SOCK_V5,rep_info,RSV];
    try!(stream.write(&buf));
    match addr {
        SocketAddr::V4(ipv4) => {
            let addr_vec = ipv4.ip().octets();
            let buf = [ATYP_IP_V4,addr_vec[3],addr_vec[2],addr_vec[1],addr_vec[0]];
            try!(stream.write(&buf));
            try!(stream.write_u16(ipv4.port()));
        },
        SocketAddr::V6(ipv6) => panic!("Found ipv6 Address"),
    }
    Ok(())
}

pub fn success_reply(stream: &mut Tcp, addr:SocketAddr) -> Result<(),TcpError> {
    return get_reply(stream,addr,1 as u8);
}

pub fn failure_reply(stream: &mut Tcp, addr:SocketAddr) -> Result<(),TcpError> {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0,0,0,0),0));
    return get_reply(stream,addr,0 as u8);
}

// #[test]
// fn test_set_u32() {
//     let mut a =[0u8;4];
//     a[0] = 0xD8 as u8;
//     a[1] = 0xEC as u8;
//     a[2] = 0xA2 as u8;
//     a[3] = 0x83 as u8;
//     let b  = 0xD8ECA283 as u32;
//     let mut aa = read_u32(a);
//     Tcp::set_u32(&mut a,aa);
//     assert_eq!(aa,b );

// }