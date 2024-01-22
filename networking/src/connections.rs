use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Packet {
    pub addr: SocketAddr,
    pub data: Vec<u8>,
}

enum TcpConnEvent {
    New {
        addr: SocketAddr,
        recv: Receiver<Vec<u8>>,
        send: Sender<Vec<u8>>,
    },
    Killed {
        addr: SocketAddr,
    },
}

pub(crate) struct FramedTcpReceiver {
    last_frame_size: u32,
    buf: Vec<u8>,
}

#[allow(clippy::type_complexity)]
pub struct Connections {
    udp_send: Sender<Packet>,
    udp_recv: Receiver<Packet>,

    tcp_conn_events: Receiver<TcpConnEvent>,
    tcp_conns: HashMap<SocketAddr, (FramedTcpReceiver, Receiver<Vec<u8>>, Sender<Vec<u8>>)>,
}

#[derive(Debug)]
pub enum ConnectionsError {
    UdpBind(std::io::Error),
    TcpBind(std::io::Error),
}

impl Connections {
    pub fn new(addr: SocketAddr) -> Result<Self, ConnectionsError> {
        let udp_sock = UdpSocket::bind(addr).map_err(ConnectionsError::UdpBind)?;
        let (udp_send_conn, udp_recv) = channel();
        let (udp_send, udp_recv_conn) = channel();

        let (tcp_conn_events_send, tcp_conn_events_recv) = channel();

        let tcp_listener = TcpListener::bind(addr).map_err(ConnectionsError::TcpBind)?;

        Self::start_process_threads(
            udp_sock,
            udp_recv,
            udp_send,
            tcp_listener,
            tcp_conn_events_send,
        );

        Ok(Self {
            udp_send: udp_send_conn,
            udp_recv: udp_recv_conn,
            tcp_conn_events: tcp_conn_events_recv,
            tcp_conns: HashMap::new(),
        })
    }

    pub fn send_udp(&self, addr: SocketAddr, data: Vec<u8>) -> Option<()> {
        self.udp_send.send(Packet { addr, data }).ok()
    }

    pub fn recv_udp(&self) -> Option<Packet> {
        self.udp_recv.try_recv().ok()
    }

    pub fn send_tcp(&self, addr: SocketAddr, frame: Vec<u8>) -> Option<()> {
        if let Some((_, _, send)) = self.tcp_conns.get(&addr) {
            return send.send(frame).ok();
        }
        None
    }

    pub fn remove_tcp(&mut self, addr: SocketAddr) {
        self.tcp_conns.remove(&addr);
    }

    // returns new and deleted conns
    pub fn handle_tcp_conns(&mut self) -> (Vec<SocketAddr>, Vec<SocketAddr>) {
        let mut newconns = vec![];
        let mut deletedconns = vec![];
        for event in self.tcp_conn_events.try_iter() {
            match event {
                TcpConnEvent::New { send, recv, addr } => {
                    self.tcp_conns
                        .insert(addr, (FramedTcpReceiver::new(), recv, send));
                    newconns.push(addr);
                }
                TcpConnEvent::Killed { addr } => {
                    deletedconns.push(addr);
                    self.tcp_conns.remove(&addr);
                }
            }
        }
        (newconns, deletedconns)
    }

    pub fn recv_tcp(&mut self) -> Vec<Packet> {
        let mut packets = Vec::new();
        for (addr, (frame, recv, _)) in self.tcp_conns.iter_mut() {
            let Ok(v) = recv.try_recv() else { continue };

            frame.recv(&v, |d| {
                packets.push(Packet {
                    addr: *addr,
                    data: d,
                })
            });
        }
        packets
    }

    fn start_process_threads(
        udp_sock: UdpSocket,
        udp_recv: Receiver<Packet>,
        udp_send: Sender<Packet>,
        tcp: TcpListener,
        tcp_conn_events_send: Sender<TcpConnEvent>,
    ) {
        let udp_sock_cpy = udp_sock.try_clone().unwrap();
        std::thread::spawn(move || {
            while let Ok(Packet { addr, data }) = udp_recv.recv() {
                match udp_sock_cpy.send_to(&data, addr) {
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
                match udp_sock.recv_from(&mut buf) {
                    Ok((size, addr)) => {
                        let data = buf[..size].to_vec();
                        udp_send.send(Packet { addr, data }).unwrap();
                    }
                    Err(e) => {
                        log::error!("udp recv error: {}", e);
                    }
                }
            }
        });

        std::thread::spawn(move || loop {
            match tcp.accept() {
                Ok((mut stream, addr)) => {
                    let _ = stream.set_nodelay(true);

                    let (send_from_tcp, recv) = channel::<Vec<u8>>();
                    let (send, recv_from_tcp) = channel::<Vec<u8>>();

                    let mut stream_cpy = stream.try_clone().unwrap();
                    let send_cpy = tcp_conn_events_send.clone();

                    std::thread::spawn(move || {
                        let mut buf = [0u8; 65536];
                        loop {
                            match stream_cpy.read(&mut buf) {
                                Ok(size) => {
                                    let data = buf[..size].to_vec();
                                    send_from_tcp.send(data).unwrap();
                                }
                                Err(e) => {
                                    log::error!("tcp recv error: {}", e);
                                    let _ = send_cpy.send(TcpConnEvent::Killed { addr });
                                    break;
                                }
                            }
                        }
                    });

                    std::thread::spawn(move || loop {
                        match recv_from_tcp.recv() {
                            Ok(data) if !data.is_empty() => {
                                match stream.write_all(&(data.len() as u32).to_le_bytes()) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        break;
                                    }
                                }
                                match stream.write_all(&data) {
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

                    let _ = tcp_conn_events_send.send(TcpConnEvent::New { addr, recv, send });
                }
                Err(e) => {
                    log::error!("tcp accept error: {}", e);
                }
            }
        });
    }
}

impl FramedTcpReceiver {
    pub fn new() -> Self {
        Self {
            last_frame_size: 0,
            buf: Vec::with_capacity(65565),
        }
    }

    pub fn recv(&mut self, mut data: &[u8], mut frame_callback: impl FnMut(Vec<u8>)) {
        loop {
            if self.last_frame_size == 0 {
                if data.len() + self.buf.len() < 4 {
                    self.buf.extend(data);
                    return;
                }

                let mut size_bytes = [0u8; 4];
                let buf_len = self.buf.len();
                size_bytes[..buf_len].copy_from_slice(&self.buf);
                size_bytes[buf_len..4].copy_from_slice(&data[..4 - buf_len]);
                self.last_frame_size = u32::from_le_bytes(size_bytes);
                self.buf.clear();

                data = &data[4 - buf_len..];
            }

            let data_remaining = self.last_frame_size as usize - self.buf.len();
            if data.len() < data_remaining {
                self.buf.extend(data);
                return;
            }

            let mut frame = Vec::with_capacity(self.last_frame_size as usize);
            frame.extend(&self.buf);
            self.buf.clear();
            frame.extend(&data[..data_remaining]);
            data = &data[data_remaining..];
            self.last_frame_size = 0;
            frame_callback(frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_framed_tcp_receiver() {
        let mut receiver = FramedTcpReceiver::new();

        let mut data = vec![0u8; 1000];
        data[0..4].copy_from_slice(&996u32.to_le_bytes());
        data[4..].copy_from_slice(&vec![1u8; 996]);

        receiver.recv(&data, |d| assert_eq!(d, vec![1u8; 996]));
        receiver.recv(&[0u8; 0], |_| panic!("should not be called"));

        receiver = FramedTcpReceiver::new();

        let s = 5u32.to_le_bytes();
        receiver.recv(&s[..2], |_| panic!("should not be called"));
        receiver.recv(&s[2..], |_| panic!("should not be called"));
        receiver.recv(&[1u8; 3], |_| panic!("should not be called"));
        receiver.recv(&[2u8; 3], |d| assert_eq!(d, vec![1u8, 1u8, 1u8, 2u8, 2u8]));
        assert_eq!(receiver.buf, vec![2u8]);
        assert_eq!(receiver.last_frame_size, 0);

        receiver = FramedTcpReceiver::new();
        let mut i = 0;
        receiver.recv(&[3, 0, 0, 0, 1, 2, 3, 2, 0, 0, 0, 1, 5, 3, 0, 0, 0], |d| {
            if i == 0 {
                assert_eq!(d, vec![1, 2, 3])
            } else if i == 1 {
                assert_eq!(d, vec![1, 5])
            } else {
                panic!("should not be called");
            }
            i += 1;
        });
        assert_eq!(receiver.buf, vec![0u8; 0]);
        assert_eq!(receiver.last_frame_size, 3);
    }
}
