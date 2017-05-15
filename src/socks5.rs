//reference 
//http://wsfdl.com/python/2016/08/19/SS5_protocol.html
use std::net::{IpAddr, Ipv4Addr,Shutdown,TcpStream};
use std::io::Error;
use std::io::{Read,Write};

const SOCK_V5:u8 = 5;
const RSV: u8 = 0;
const ATYP_IP_V4: u8 = 1;
const ATYP_COMAINNAME: u8 = 3;
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

    pub fn read_u8(&mut self) -> Result<u8, TcpError> {
        let mut buf = [0u8];
        try!(self.read_buf(&mut buf));
        return Ok(buf[0]);

    }
}

fn check_method(stream: &mut Tcp) -> Result<u8,TcpError> {
    
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