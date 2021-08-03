mod utils;

use nix::net::if_::InterfaceFlags;
use nix::sys::socket::{AddressFamily, SockAddr};
use std::net;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use thiserror::Error;

const NCCL_PTR_HOST: i32 = 1;
const NCCL_PTR_CUDA: i32 = 2;

pub struct BaguaNetBackend {
    pub socket_devs: Vec<NCCLSocketDev>,
}

pub struct NCCLNetProperties {
    pub name: String,
    pub pci_path: String,
    pub guid: u64,
    pub ptr_support: i32, // NCCL_PTR_HOST or NCCL_PTR_HOST|NCCL_PTR_CUDA
    pub speed: i32,       // Port speed in Mbps.
    pub port: i32,
    pub max_comms: i32,
}

pub struct SocketHandle {
    pub addr: net::SocketAddr,
}

pub struct SocketListenComm {
    pub tcp_listener: net::TcpListener,
}

pub struct SocketSendComm {
    pub tcp_stream: Arc<Mutex<net::TcpStream>>,
}

pub struct SocketRecvComm {
    pub tcp_stream: net::TcpStream,
    pub addr: net::SocketAddr,
}

#[derive(Error, Debug)]
pub enum BaguaNetError {
    #[error("io error")]
    IOError(String),
}

impl BaguaNetBackend {
    const DEFAULT_SOCKET_MAX_COMMS: i32 = 65536;

    pub fn new() -> BaguaNetBackend {
        Self {
            socket_devs: find_interfaces(),
        }
    }

    pub fn devices(&self) -> usize {
        self.socket_devs.len()
    }

    pub fn get_device_properties(&self, dev_id: usize) -> NCCLNetProperties {
        let socket_dev = &self.socket_devs[dev_id];

        NCCLNetProperties {
            name: socket_dev.interface_name.clone(),
            pci_path: socket_dev.pci_path.clone(),
            guid: dev_id as u64,
            ptr_support: NCCL_PTR_HOST,
            speed: utils::get_net_if_speed(&socket_dev.interface_name),
            port: 0,
            max_comms: BaguaNetBackend::DEFAULT_SOCKET_MAX_COMMS,
        }
    }

    pub fn listen(&self, dev_id: usize) -> (SocketHandle, Arc<SocketListenComm>) {
        let socket_dev = &self.socket_devs[dev_id];
        let listener = net::TcpListener::bind(socket_dev.addr.clone().to_str()).unwrap();

        (
            SocketHandle {
                addr: listener.local_addr().unwrap(),
            },
            Arc::new(SocketListenComm {
                tcp_listener: listener,
            }),
        )
    }

    pub fn connect(&self, _dev_id: usize, socket_handle: SocketHandle) -> SocketSendComm {
        let stream = net::TcpStream::connect(socket_handle.addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        SocketSendComm { tcp_stream: Arc::new(Mutex::new(stream)) }
    }

    pub fn accept(&self, listen_comm: Arc<SocketListenComm>) -> Arc<SocketRecvComm> {
        let (stream, addr) = listen_comm.tcp_listener.accept().unwrap();

        Arc::new(SocketRecvComm {
            tcp_stream: stream,
            addr: addr,
        })
    }

    pub fn isend(send_comm: SocketSendComm, data: &[u8]) -> Result<(), BaguaNetError> {
        send_comm.tcp_stream.lock().unwrap().write(data);
        Ok(())
    }
}

#[derive(Debug)]
pub struct NCCLSocketDev {
    pub interface_name: String,
    pub addr: SockAddr,
    pub pci_path: String,
}

pub fn find_interfaces() -> Vec<NCCLSocketDev> {
    let mut socket_devs = Vec::<NCCLSocketDev>::new();
    const MAX_IF_NAME_SIZE: usize = 16;
    // TODO: support user specified interfaces
    let addrs = nix::ifaddrs::getifaddrs().unwrap();
    for ifaddr in addrs {
        match ifaddr.address {
            Some(addr) => {
                println!("interface {} address {}", ifaddr.interface_name, addr);

                if addr.family() != AddressFamily::Inet && addr.family() != AddressFamily::Inet6 {
                    continue;
                }

                if ifaddr.flags.contains(InterfaceFlags::IFF_LOOPBACK) {
                    continue;
                }

                assert_eq!(ifaddr.interface_name.len() < MAX_IF_NAME_SIZE, true);

                let found_ifs: Vec<&NCCLSocketDev> = socket_devs
                    .iter()
                    .filter(|scoket_dev| scoket_dev.interface_name == ifaddr.interface_name)
                    .collect();
                if found_ifs.len() > 0 {
                    continue;
                }

                socket_devs.push(NCCLSocketDev {
                    addr: addr,
                    interface_name: ifaddr.interface_name.clone(),
                    pci_path: format!("/sys/class/net/{}/device", ifaddr.interface_name),
                })
            }
            None => {
                println!(
                    "interface {} with unsupported address family",
                    ifaddr.interface_name
                );
            }
        }
    }

    socket_devs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bagua_net = BaguaNetBackend::new();
        println!("bagua_net.socket_devs={:?}", bagua_net.socket_devs);

        assert_eq!(2 + 2, 4);
    }
}
