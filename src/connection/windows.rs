use std::path::PathBuf;

use named_pipe::PipeClient;

use super::base::Connection;
use crate::Result;

pub struct Socket {
    socket: PipeClient,
}

impl Connection for Socket {
    type Socket = PipeClient;

    fn connect() -> Result<Self> {
        // TODO: Add timed out reads and writes to 16s
        let mut socket = PipeClient::connect(Self::socket_path(0))?;
        socket.set_read_timeout(Some(Self::READ_WRITE_TIMEOUT));
        socket.set_write_timeout(Some(Self::READ_WRITE_TIMEOUT));
        Ok(Self { socket })
    }

    fn ipc_path() -> PathBuf {
        PathBuf::from(r"\\.\pipe\")
    }

    fn socket(&mut self) -> &mut Self::Socket {
        &mut self.socket
    }
}
