use crate::error::Error;
use crate::requested_transport::*;
use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use stun::attribute::*;
use stun::message::*;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};

const DEFAULT_RTO_IN_MS: u16 = 200;
const MAX_DATA_BUFFER_SIZE: usize = u16::MAX as usize; // message size limit for Chromium
const MAX_READ_QUEUE_SIZE: usize = 1024;

pub struct ClientConfig {
    pub turn_server_address: String,
    pub username: String,
    pub password: String,
    pub connection: Arc<UdpSocket>,
}

pub struct Client {
    client_internal: Arc<Mutex<ClientInternal>>,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self> {
        let client_internal = ClientInternal::new(config).await?;
        Ok(Client {
            client_internal: Arc::new(Mutex::new(client_internal)),
        })
    }
    pub async fn listen(&self) -> Result<()> {
        let client_internal = self.client_internal.lock().await;
        client_internal.listen().await
    }
    pub async fn allocate(&self) -> Result<()> {
        let config = {
            let mut client_internal = self.client_internal.lock().await;
            client_internal.allocate().await?;
        };
        Ok(())
    }
}

struct ClientInternal {
    connection: Arc<UdpSocket>,
    turn_server_address: String,
    username: String,
    password: String,
}
impl ClientInternal {
    async fn new(config: ClientConfig) -> Result<Self> {
        // 受け取ったconfigを使ってClientInternalを作る
        Ok(ClientInternal {
            connection: Arc::clone(&config.connection),
            turn_server_address: config.turn_server_address,
            username: config.username,
            password: config.password,
        })
    }

    async fn listen(&self) -> Result<()> {
        let connection = Arc::clone(&self.connection);

        println!("listen...");
        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_DATA_BUFFER_SIZE];
            loop {
                let (n, from) = match connection.recv_from(&mut buf).await {
                    Ok((n, from)) => (n, from),
                    Err(err) => {
                        log::debug!("{}", err);
                        break;
                    }
                };
                log::debug!("received {} bytes of udp from {}", n, from);
                // handle inbound packet
            }
        });
        Ok(())
    }

    async fn allocate(&self) -> Result<()> {
        let (result_ch_tx, mut result_ch_rx): (
            Option<mpsc::Sender<MpscResult>>,
            Option<mpsc::Receiver<MpscResult>>,
        ) = {
            let (tx, rx) = mpsc::channel(1);
            (Some(tx), Some(rx))
        };

        let mut allocate_request_message = Message::new(METHOD_ALLOCATE, CLASS_REQUEST);
        let requested_transport = RequestedTransport {
            protocol: PROTO_UDP,
        };
        println!("allocate_request_message: {:?}", allocate_request_message);
        allocate_request_message.set_extra_attribute(Box::new(requested_transport))?;
        println!("allocate_request_message: {:?}", allocate_request_message);

        let allocate_request_message_packet = &allocate_request_message.encode_to_packet();
        println!(
            "allocate_request_message_packet: {:?}",
            allocate_request_message_packet
        );
        // send message
        self.connection
            .send_to(&allocate_request_message_packet, &self.turn_server_address)
            .await?;
        println!("allocate_request_message_packet sent!");

        let received_message = if let Some(mut result_ch_rx) = result_ch_rx {
            match result_ch_rx.recv().await {
                Some(tr) => Ok(tr),
                None => Err(Error::ErrReceiverClosed),
            }
        } else {
            Err(Error::ErrReceiverClosed)
        };
        println!("received Message! => {:?}", received_message);

        // この部分はlisten部分でやりたい
        // let mut buf = [0; 100];
        // let (n, addr) = match self.connection.recv_from(&mut buf).await {
        //     Ok((n, addr)) => (n, addr),
        //     Err(err) => {
        //         panic!("{:?}", err);
        //     }
        // };
        // let message = Message::decode_from_packet(&buf[..n].to_vec()).unwrap();
        // println!("received packet: {:?}", message);
        // match message.attributes.iter().find(|e| e.typ == ATTR_ERROR_CODE) {
        //     Some(attribute) => attribute,
        //     None => {
        //         return Err(Error::ErrAllocateResponseIncludeNoErrorCodeAttribute.into());
        //     }
        // };
        // // attributesの中に ERROCODE attirbute unauthorisedが返ってくるはず
        // // send authorised message
        // let mut second_allocate_request_message = Message::new(METHOD_ALLOCATE, CLASS_REQUEST);
        // let requested_transport = RequestedTransport {
        //     protocol: PROTO_UDP,
        // };
        // second_allocate_request_message.set_extra_attribute(Box::new(requested_transport))?;
        // // set user
        // let username = "uchida00";
        // let user = Attribute::new(ATTR_USERNAME, 8, username.as_bytes().to_vec());
        // second_allocate_request_message.attributes.push(user);

        // // 受け取ったnonceを使う set nonce
        // let nonce = match message.attributes.iter().find(|e| e.typ == ATTR_NONCE) {
        //     Some(attribute) => attribute,
        //     None => {
        //         return Err(Error::ErrAllocateResponseIncludeNoErrorCodeAttribute.into());
        //     }
        // };
        // second_allocate_request_message
        //     .attributes
        //     .push(nonce.clone());
        // println!(
        //     "second_allocate_request_message: {:?}",
        //     second_allocate_request_message
        // );

        // // 受け取ったrealmを使う set realm
        // let realm = match message.attributes.iter().find(|e| e.typ == ATTR_REALM) {
        //     Some(attribute) => attribute,
        //     None => {
        //         return Err(Error::ErrAllocateResponseIncludeNoErrorCodeAttribute.into());
        //     }
        // };
        // second_allocate_request_message
        //     .attributes
        //     .push(realm.clone());

        // // set message integrity
        // // set fingerprint
        // let second_allocate_request_message_packet =
        //     &second_allocate_request_message.encode_to_packet();
        // println!(
        //     "second_allocate_request_message_packet: {:?}",
        //     second_allocate_request_message_packet
        // );
        // self.connection
        //     .send_to(
        //         &second_allocate_request_message_packet,
        //         &self.turn_server_address,
        //     )
        //     .await?;

        // let mut buf = [0; 100];
        // println!("sended");
        // let (n, addr) = match self.connection.recv_from(&mut buf).await {
        //     Ok((n, addr)) => (n, addr),
        //     Err(err) => {
        //         panic!("{:?}", err);
        //     }
        // };
        // println!("{:?} {:?}", n, addr);

        Ok(())
    }
}

#[derive(Debug)] //Clone
pub struct MpscResult {
    pub msg: Message,
    pub from: SocketAddr,
    pub retries: u16,
    pub err: Option<Error>,
}

impl Default for MpscResult {
    fn default() -> Self {
        MpscResult {
            msg: Message::default(),
            from: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
            retries: 0,
            err: None,
        }
    }
}
