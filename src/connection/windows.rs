use super::base::Connection;
use crate::Result;
use std::{path::PathBuf, time};

pub struct Socket {
    socket: std::fs::File,
}

impl Connection for Socket {
    type Socket = std::fs::File;

    fn connect() -> Result<Self> {
        // TODO: Add timed out reads and writes to 16s
        let socket = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(Self::socket_path(0))?;
        Ok(Self { socket })
    }

    fn ipc_path() -> PathBuf {
        PathBuf::from(r"\\.\pipe\")
    }

    fn socket(&mut self) -> &mut Self::Socket {
        &mut self.socket
    }
}
