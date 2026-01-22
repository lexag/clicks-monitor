use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, Instant},
};

use chrono::Utc;
use common::{
    mem::{
        network::{ConnectionEnd, ConnectionInfo, IpAddress, SubscriberInfo},
        str::StaticString,
        typeflags::{MessageType, RequestType},
    },
    protocol::{
        message::{LargeMessage, Message, SmallMessage},
        request::Request,
    },
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use local_ip_address::local_ip;
#[derive(Debug)]
pub struct UdpClient {
    pub local: ConnectionInfo,
    socket: UdpSocket,
    local_tx: Sender<(Message, usize)>,
    local_rx: Receiver<(Message, usize)>,
    pub active: bool,
    pub rx_message_tally: HashMap<MessageType, (usize, usize)>,
    pub tx_message_tally: HashMap<RequestType, (usize, usize)>,
}

impl UdpClient {
    pub fn new() -> UdpClient {
        let (tx, rx): (Sender<(Message, usize)>, Receiver<(Message, usize)>) = unbounded();
        UdpClient {
            rx_message_tally: HashMap::new(),
            tx_message_tally: HashMap::new(),
            local: ConnectionInfo {
                end: ConnectionEnd::Local,
                address: IpAddress::new([0, 0, 0, 0], 0),
                identifier: StaticString::new(&whoami::devicename()),
                ..Default::default()
            },
            socket: UdpSocket::bind(
                local_ip_address::local_ip()
                    .expect("ip address kinda needed")
                    .to_string()
                    + ":0",
            )
            .unwrap(),
            local_tx: tx,
            local_rx: rx,
            active: false,
        }
    }

    pub fn connect(
        &mut self,
        identifier: StaticString<32>,
        address: IpAddress,
    ) -> Result<ConnectionInfo, std::io::Error> {
        self.socket.connect(address.to_string())?;
        self.send_msg(Request::Subscribe(SubscriberInfo {
            identifier,
            address: IpAddress::from_address_str(&self.get_local_address().to_string())
                .expect("pls"),
            message_kinds: MessageType::CueData
                | MessageType::ShowData
                | MessageType::TransportData
                | MessageType::TimecodeData
                | MessageType::BeatData
                | MessageType::NetworkChanged
                | MessageType::JACKStateChanged
                | MessageType::ConfigurationChanged
                | MessageType::ShutdownOccured
                | MessageType::Heartbeat,

            last_contact: Utc::now().timestamp() as u128,
        }));
        let ci = ConnectionInfo {
            identifier: StaticString::new(""),
            address: IpAddress::from_address_str(&self.socket.peer_addr().unwrap().to_string())
                .expect("pls"),
            end: ConnectionEnd::Remote,
        };
        Ok(ci)
    }

    pub fn get_receiver(&self) -> Receiver<(Message, usize)> {
        self.local_rx.clone()
    }

    pub fn get_local_address(&self) -> std::net::SocketAddr {
        self.socket.local_addr().unwrap()
    }

    pub fn start(&mut self) {
        if let Ok(_addr) = self.socket.peer_addr() {
            return;
        }

        self.local.address =
            IpAddress::from_str_and_port(&local_ip().unwrap().to_string(), 0).unwrap_or_default();
        self.socket = UdpSocket::bind(format!("{}", self.local.address)).unwrap();
        self.local.address.port = self.socket.local_addr().map_or(0, |f| f.port());
        println!(
            "Local address: {:?}\nConnection info: {:?}\nRemote address: {:?}",
            self.socket,
            self.local,
            self.socket.peer_addr()
        );

        let socket = self.socket.try_clone().unwrap();
        let tx = self.local_tx.clone();
        let mut last_ping_time = Instant::now();
        let mut last_recv_time = Instant::now();
        let mut buf = [0u8; 65536];

        std::thread::spawn(move || loop {
            //println!("udp loop");
            //println!(
            //    "Local address: {:?}\nRemote address: {:?}",
            //    socket,
            //    socket.peer_addr()
            //);
            if last_ping_time.elapsed() > Duration::from_secs(600) {
                Self::anonymous_send(&socket, Request::Ping);
                last_ping_time = Instant::now()
            }
            if last_recv_time.elapsed() > Duration::from_secs(10) {
                let _ = tx.try_send((Message::Small(SmallMessage::ShutdownOccured), 1));
            }
            buf.fill(0);
            match socket.recv(&mut buf) {
                Err(e) => println!("recv function failed: {e:?}"),
                Ok(packet_len) => {
                    last_recv_time = Instant::now();
                    //println!("Receiving message! ({} bytes)", packet_len);
                    if buf[0] == 0xD2 {
                        match postcard::from_bytes::<LargeMessage>(&buf[1..]) {
                            Ok(msg) => {
                                //println!("It was a large message: {:?}", msg);
                                let _ = tx.try_send((Message::Large(msg), packet_len));
                            }
                            Err(err) => {
                                panic!(
                                    "failed parse! \n {:#02X?}...\n({} bytes)\n{:?}",
                                    &buf[..packet_len + 5],
                                    packet_len,
                                    err
                                )
                            }
                        }
                    } else if buf[0] == 0xE1 {
                        match postcard::from_bytes::<SmallMessage>(&buf[1..]) {
                            Ok(msg) => {
                                //println!("It was a small message: {:?}", msg);
                                let _ = tx.try_send((Message::Small(msg), packet_len));
                            }
                            Err(err) => {
                                panic!(
                                    "failed parse! \n {:#02X?}...\n({} bytes)\n{:?}",
                                    &buf[..packet_len + 5],
                                    packet_len,
                                    err
                                );
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn anonymous_send(socket: &UdpSocket, msg: Request) -> usize {
        let mut buf = [0u8; size_of::<Request>()];
        let res = postcard::to_slice(&msg, &mut buf).unwrap_or_default();
        let _ = socket.send(&res);
        res.len()
    }

    pub fn send_msg(&mut self, msg: Request) {
        println!("Sending request: {:?}", msg);
        let len = Self::anonymous_send(&self.socket, msg);
        let tally_pre = self.tx_message_tally.get(&msg.to_type()).unwrap_or(&(0, 0));
        self.tx_message_tally
            .insert(msg.to_type(), (tally_pre.0 + 1, tally_pre.1 + len));
    }
}
