use super::base::Connection;
use crate::Result;
use std::{env, net::Shutdown, os::unix::net::UnixStream, path::PathBuf, time};

pub struct Socket {
    socket: UnixStream,
}

impl Connection for Socket {
    type Socket = UnixStream;

    fn connect() -> Result<Self> {
        let connection_name = Self::socket_path(0);
        let socket = UnixStream::connect(connection_name)?;
        socket.set_nonblocking(true)?;
        socket.set_read_timeout(Some(Self::READ_WRITE_TIMEOUT));
        socket.set_write_timeout(Some(Self::READ_WRITE_TIMEOUT));
        Ok(Self { socket })
    }

    fn ipc_path() -> PathBuf {
        let tmp = env::var("XDG_RUNTIME_DIR")
            .or_else(|_| env::var("TMPDIR"))
            .or_else(|_| match env::temp_dir().to_str() {
                None => Err("Failed to convert temp_dir"),
                Some(tmp) => Ok(tmp.to_owned()),
            })
            .unwrap_or_else(|_| "/tmp".to_owned());
        PathBuf::from(tmp)
    }

    fn socket(&mut self) -> &mut Self::Socket {
        &mut self.socket
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        if self.socket.shutdown(Shutdown::Both).is_err() {
            error!("Failed to properly shut down socket");
        }
    }
}
