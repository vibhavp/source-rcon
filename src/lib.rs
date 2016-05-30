extern crate byteorder;

use std::sync::{Mutex, Arc};
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering::SeqCst;
use std::string::String;
use std::io::prelude::*;
use std::io;
use std::io::Cursor;
use std::net::{TcpStream, ToSocketAddrs};
use std::result::Result;
use std::option::Option;
use std::time::Duration;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};

pub mod error;
use error::Error;

pub struct RCON {
    stream: Arc<Mutex<TcpStream>>,
    last_id: AtomicIsize,
}

pub struct Response {
    pub req_id: i32,
    pub packet_type: i32,
    pub body: String,
}

pub const SERVERDATA_AUTH: i32 = 3;
pub const SERVERDATA_AUTH_RESPONSE: i32 = 2;
pub const SERVERDATA_EXECCOMMAND: i32 = 2;
pub const SERVERDATA_RESPONSE_VALUE: i32 = 0;

impl RCON {
    pub fn new<A: ToSocketAddrs>(addr: A, pwd: &str) -> Result<RCON, Error> {
        RCON::with_timeout(addr, pwd, Some(Duration::from_secs(5)),
                           Some(Duration::from_secs(5)))
    }

    pub fn with_timeout<A: ToSocketAddrs>(addr: A, pwd: &str,
                                          read: Option<Duration>,
                                          write: Option<Duration>) -> Result<RCON, Error> {
        let stream = try!(TcpStream::connect(addr).map_err(|e| {Error::IOError(e)}));
        try!(stream.set_read_timeout(read).map_err(|e| Error::IOError(e)));
        try!(stream.set_write_timeout(write).map_err(|e| Error::IOError(e)));
        RCON::from_stream(stream, pwd)
    }

    fn from_stream(stream: TcpStream, pwd: &str) -> Result<RCON, Error> {
        let mut rcon = RCON{stream: Arc::new(Mutex::new(stream)), last_id: AtomicIsize::new(1)};
        try!(rcon.write(pwd, SERVERDATA_AUTH).map_err(|e| {Error::IOError(e)}));
        let resp = try!(rcon.read_response());

        if resp.req_id != 1 {
            Err(Error::InvalidPacket)
        } else {
            Ok(rcon)
        }
    }

    fn write(&mut self, body: &str, packet_type: i32) -> Result<i32, io::Error> {
        let reqid = self.last_id.fetch_add(1, SeqCst) as i32;

        let mut w = Vec::with_capacity(10);
        try!(w.write_i32::<LittleEndian>(10 + body.len() as i32));
        try!(w.write_i32::<LittleEndian>(reqid as i32));
        try!(w.write_i32::<LittleEndian>(packet_type));
        w.append(&mut Vec::from(body.as_bytes()));
        try!(w.write_u8(0));
        try!(w.write_u8(0));

        self.last_id.compare_and_swap(isize::max_value(), 1, SeqCst);
        let data = self.stream.clone();
        let mut stream = data.lock().unwrap();

        try!(stream.write(w.as_slice()));
        Ok(reqid)
    }

    /// Write a query to the connected server.
    /// A successful write returns `Result::Ok(n)` where `n` is the request id
    /// for the sent query.
    #[inline]
    pub fn write_cmd(&mut self, cmd: &str) -> Result<i32, io::Error> {
        self.write(cmd, SERVERDATA_EXECCOMMAND)
    }

    pub fn read_response(&mut self) -> Result<Response, Error> {
        let data = self.stream.clone();
        let mut stream = data.lock().unwrap();
        let mut buf = Vec::with_capacity(10);
        println!("reading");

        try!(stream.read_to_end(&mut buf).map_err(|e| {Error::IOError(e)}));
        println!("done");
        let mut rdr = Cursor::new(buf);
        let size = rdr.read_i32::<LittleEndian>().unwrap();
        if size < 10 {
            return Err(Error::InvalidPacket);
        }

        let req_id = rdr.read_i32::<LittleEndian>().unwrap();
        let packet_type = rdr.read_i32::<LittleEndian>().unwrap();
        let mut body = Vec::new();

        try!(rdr.read_until(b'\0', &mut body).map_err(|e| {Error::IOError(e)}));

        Ok(Response {
            packet_type: packet_type,
            req_id: req_id,
            body: try!(std::string::String::from_utf8(body).map_err(|e| Error::UTF8Error(e))),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use super::RCON;
        RCON::new("stadiumchi2.game.nfoservers.com:27015", "dU8rcon_wWe2").unwrap();
    }
}
