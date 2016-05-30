use std::io;
use std::string;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    UTF8Error(string::FromUtf8Error),
    AuthError,
    InvalidPacket,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error::IOError(ref e)   => write!(f, "{}", e),
            &Error::UTF8Error(ref e) => write!(f, "{}", e),
            &Error::AuthError => write!(f, "Error::AuthError"),
            &Error::InvalidPacket => write!(f, "Error::InvalidPacket"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::IOError(ref e)   => e.description(),
            &Error::UTF8Error(ref e) => e.description(),
            &Error::AuthError => "authentication failed",
            &Error::InvalidPacket => "invalid packet"
        }
    }
}
