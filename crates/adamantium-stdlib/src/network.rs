//! Bounded TCP networking primitives.

use crate::{Error, Result};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

fn first_address(address: impl ToSocketAddrs) -> Result<SocketAddr> {
    address
        .to_socket_addrs()
        .map_err(|error| Error::new("network.resolve", error))?
        .next()
        .ok_or_else(|| Error::new("network.resolve", "address resolved to no endpoints"))
}

pub fn tcp_exchange(
    address: impl ToSocketAddrs,
    payload: &[u8],
    timeout: Duration,
    maximum_response_bytes: usize,
) -> Result<Vec<u8>> {
    let address = first_address(address)?;
    let mut stream = TcpStream::connect_timeout(&address, timeout)
        .map_err(|error| Error::new("network.connect", error))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| Error::new("network.timeout", error))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| Error::new("network.timeout", error))?;
    stream
        .write_all(payload)
        .map_err(|error| Error::new("network.write", error))?;
    let mut response = Vec::new();
    stream
        .take(maximum_response_bytes as u64)
        .read_to_end(&mut response)
        .map_err(|error| Error::new("network.read", error))?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_address_is_reported() {
        let error =
            tcp_exchange("not a socket address", b"", Duration::from_millis(1), 1).unwrap_err();
        assert_eq!(error.operation, "network.resolve");
    }
}
