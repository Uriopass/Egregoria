use crate::connections::{ConnectionsError, FramedTcpReceiver};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

pub struct ConnectionClient {
    udp_send: Sender<Vec<u8>>,
    udp_recv: Receiver<Vec<u8>>,

    tcp_conn: (FramedTcpReceiver, Receiver<Vec<u8>>, Sender<Vec<u8>>),
    disconnected: Arc<AtomicBool>,
}

impl ConnectionClient {
    pub fn new(addr: SocketAddr) -> Result<Self, ConnectionsError> {
        let udp_sock = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))
            .map_err(ConnectionsError::UdpBind)?;

        let (udp_send_conn, udp_recv) = channel();
        let (udp_send, udp_recv_conn) = channel();

        udp_sock.connect(addr).map_err(ConnectionsError::UdpBind)?;

        let tcp_stream = TcpStream::connect(addr).map_err(ConnectionsError::TcpBind)?;

        let _ = tcp_stream.set_nodelay(true);

        let (send_from_tcp, recv) = channel::<Vec<u8>>();
        let (send, recv_from_tcp) = channel::<Vec<u8>>();

        let disconnected = Arc::new(AtomicBool::new(false));

        Self::start_process_threads(
            udp_sock,
            udp_recv,
            udp_send,
            tcp_stream,
            send_from_tcp,
            recv_from_tcp,
            disconnected.clone(),
        );

        Ok(Self {
            udp_send: udp_send_conn,
            udp_recv: udp_recv_conn,
            tcp_conn: (FramedTcpReceiver::new(), recv, send),
            disconnected,
        })
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::SeqCst)
    }

    pub fn send_udp(&self, data: Vec<u8>) -> Option<()> {
        self.udp_send.send(data).ok()
    }

    pub fn recv_udp(&self) -> Option<Vec<u8>> {
        self.udp_recv.try_recv().ok()
    }

    pub fn send_tcp(&self, data: Vec<u8>) -> Option<()> {
        self.tcp_conn.2.send(data).ok()
    }

    pub fn recv_tcp(&mut self) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        let Ok(v) = self.tcp_conn.1.try_recv() else {
            return packets;
        };

        self.tcp_conn.0.recv(&v, |d| {
            packets.push(d);
        });
        packets
    }

    fn start_process_threads(
        udp_sock: UdpSocket,
        udp_recv: Receiver<Vec<u8>>,
        udp_send: Sender<Vec<u8>>,
        mut tcp_stream: TcpStream,
        tcp_send: Sender<Vec<u8>>,
        tcp_recv: Receiver<Vec<u8>>,
        disconnected: Arc<AtomicBool>,
    ) {
        let udp_sock_cpy = udp_sock.try_clone().unwrap();
        std::thread::spawn(move || {
            while let Ok(data) = udp_recv.recv() {
                match udp_sock_cpy.send(&data) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("udp send error: {}", e);
                    }
                }
            }
        });

        std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            loop {
                match udp_sock.recv(&mut buf) {
                    Ok(size) => {
                        let data = buf[..size].to_vec();
                        udp_send.send(data).unwrap();
                    }
                    Err(e) => {
                        log::error!("udp recv error: {}", e);
                    }
                }
            }
        });

        let mut stream_cpy = tcp_stream.try_clone().unwrap();

        std::thread::spawn(move || {
            let mut buf = [0u8; 65536];
            loop {
                match stream_cpy.read(&mut buf) {
                    Ok(size) => {
                        let data = buf[..size].to_vec();
                        tcp_send.send(data).unwrap();
                    }
                    Err(e) => {
                        log::error!("tcp recv error: {}", e);
                        disconnected.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });

        std::thread::spawn(move || loop {
            match tcp_recv.recv() {
                Ok(data) if !data.is_empty() => {
                    match tcp_stream.write_all(&(data.len() as u32).to_le_bytes()) {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    }
                    match tcp_stream.write_all(&data) {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    }
                }
                Err(_) => break,
                _ => {}
            }
        });
    }
}
