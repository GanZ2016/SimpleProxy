//reference 
//http://wsfdl.com/python/2016/08/19/SS5_protocol.html
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, SocketAddr, Shutdown,TcpStream};
use std::io::Error;
use std::io::{Read,Write};

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


impl Tcp {

    pub fn read_buf(&mut self, buf: &mut [u8]) ->Result<(),TcpError>{
        let mut l = 0;
        while l < buf.len() {
            match self.stream.read(&mut buf[l..]) {
                Ok(0) => return Err(TcpError::ErrorData),
                Ok(res) => l += res,
                Err(e) => return Err(TcpError::IoError(e)),
            }
        }
        return Ok(());
    }
    pub fn read_size(&mut self, size: usize) -> Result<Vec<u8>, TcpError>{
        let mut buf = Vec::with_capacity(size);
        match self.read_buf(&mut buf) {
            Ok(expr) => return Ok(buf),
            Err(e) => return Err(e),
        }

    }

    pub fn read_u8(&mut self) -> Result<u8, TcpError> {
        let mut buf = [0u8];
        try!(self.read_buf(&mut buf));
        return Ok(buf[0]);
    }
    //u8 vector to u16
    //http://stackoverflow.com/questions/33968870/temporarily-transmute-u8-to-u16
    pub fn set_u16(a: &mut [u8], v: u16) {
        a[0] = v as u8;
        a[1] = (v >> 8) as u8;
    }

    pub fn read_u16(&mut self) -> Result<u16, TcpError> {
        let mut buf = [0u8; 2];
        try!(self.read_buf(&mut buf));
        let mut res = 0 as u16;
        Tcp::set_u16(&mut buf, res);
        return Ok(res);
    }

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

}

fn get_method(stream: &mut Tcp) -> Result<u8,TcpError> {
    
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

fn connect_target(stream: &mut Tcp) -> Result<ConnectInfo,TcpError> {
    let method = try!(get_method(stream)); // get method(0)
    stream.write(&[SOCK_V5, method]); //
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
        let mut addr_ipv4 = [0u8,4];
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
            return Ok(ConnectInfo::Unknown);
        }
    }
}