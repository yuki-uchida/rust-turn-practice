use crate::error::*;
use crate::util::Conn;
use md5::{Digest, Md5};
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use stun::attribute::{Nonce, Realm, ATTR_MESSAGE_INTEGRITY, ATTR_NONCE, ATTR_REALM};
use stun::error_code::*;
use stun::integrity::*;
use stun::message::*;
use tokio::sync::Mutex;
use tokio::time::Instant;

pub struct Request {
    conn: Arc<dyn Conn + Send + Sync>,
    packet: Vec<u8>,
    src_address: SocketAddr,
    kind: RequestType,
    pub realm: String,
    nonces: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Request {
    pub fn new(
        conn: Arc<dyn Conn + Send + Sync>,
        packet: Vec<u8>,
        addr: SocketAddr,
    ) -> Result<Self> {
        let request = if Request::is_channel_data(packet.to_vec()) {
            Request {
                conn: conn,
                packet: packet.to_vec(),
                src_address: addr,
                kind: CHANNEL_DATA,
                realm: String::new(),
                nonces: Arc::new(Mutex::new(HashMap::new())),
            }
        } else {
            Request {
                conn: conn,
                packet: packet.to_vec(),
                src_address: addr,
                kind: STUN_PACKET,
                realm: String::new(),
                nonces: Arc::new(Mutex::new(HashMap::new())),
            }
        };
        match request.kind {
            STUN_PACKET => Ok(request),
            CHANNEL_DATA => Ok(request),
            _ => Err(Error::ErrRequestTypeUnknown),
        }
    }

    // RFC 5766 sec 11.4
    // The ChannelData message is used to carry application data between the client and the server.
    pub fn is_channel_data(packet: Vec<u8>) -> bool {
        let num: u16 = u16::from_be_bytes([packet[0], packet[1]]);
        let is_channel: bool = match num {
            0b01 => true,
            _ => false,
        };
        return is_channel;
    }
    pub async fn handle_request(&mut self) -> Result<()> {
        match self.kind {
            STUN_PACKET => self.handle_turn_packet().await,
            _ => self.handle_channel_data().await,
        }
    }
    pub async fn handle_turn_packet(&mut self) -> Result<()> {
        println!("handling turn packet!");
        let mut message =
            Message::decode_from_packet(&self.packet).expect("Cant decode STUN Message");
        println!("decode from turn packet into stun mesage => {:?}", message);
        if message.class == CLASS_INDICATION {
            Ok(())
        } else if message.class == CLASS_REQUEST {
            match message.method {
                METHOD_ALLOCATE => self.handle_allocate_request(&message).await,
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
    pub async fn handle_channel_data(&mut self) -> Result<()> {
        Ok(())
    }
    pub async fn authenticate_request(
        &mut self,
        message: &Message,
        method: Method,
    ) -> Result<Option<MessageIntegrity>> {
        // RFC5389 10.2.2
        if !message.contains(ATTR_MESSAGE_INTEGRITY) {
            self.respond_with_nonce(message, method, CODE_UNAUTHORIZED)
                .await?;
            return Ok(None);
        }
        return Ok(None);
    }

    pub async fn handle_allocate_request(&mut self, message: &Message) -> Result<()> {
        println!("handling allocate message => {:?}", message);

        // 1.message_integrityの取得
        let message_integrity =
            if let Some(mi) = self.authenticate_request(message, METHOD_ALLOCATE).await? {
                mi
            } else {
                println!("no MessageIntegrity");
                return Ok(());
            };
        Ok(())
    }

    async fn respond_with_nonce(
        &mut self,
        message: &Message,
        method: Method,
        response_code: ErrorCode,
    ) -> Result<()> {
        let nonce = build_nonce()?;

        {
            // Nonce has already been taken
            let mut nonces = self.nonces.lock().await;
            if nonces.contains_key(&nonce) {
                return Err(Error::ErrDuplicatedNonce);
            }
            nonces.insert(nonce.clone(), Instant::now());
        }
        // STUNメッセージの構築
        // transaction_idは同じものを使うので、取り出す
        let transaction_id = message.transaction_id;
        // 返信する時のSTUN Message Method と Class
        let mut response_message = Message::new(METHOD_ALLOCATE, CLASS_ERROR);
        response_message.transaction_id = transaction_id;
        // ErrorCodeを入れる
        response_message.set_extra_attribute(Box::new(ErrorCodeAttribute {
            code: CODE_UNAUTHORIZED,
            reason: b"Unauthorized".to_vec(),
        }))?;
        println!(
            "adding ErrorCode to response_mesasge: {:?}",
            response_message
        );
        // nonce, realmを入れる
        response_message.set_extra_attribute(Box::new(Nonce::new(ATTR_NONCE, nonce)))?;
        println!("adding nonce to response_mesasge: {:?}", response_message);
        response_message
            .set_extra_attribute(Box::new(Realm::new(ATTR_REALM, self.realm.clone())))?;
        println!("adding realm to response_mesasge: {:?}", response_message);
        let response_message_packet = response_message.encode_to_packet();
        // メッセージの送信
        self.conn
            .send_to(&response_message_packet, self.src_address)
            .await?;
        return Ok(());
    }
}

// nonceとは、RFC2617で定義されたHTTPダイジェスト認証の際に、最初にサーバー側から送るランダム文字列である。
// ランダム文字列にユーザー名とパスワードをくっつけて、MD5でハッシュ化して送信する。
// Basic認証はユーザー名とパスワードを平文で送るが、ダイジェスト認証はハッシュ化して送るため、ユーザー名とパスワードを復号するのが困難。
// ただし、Basic認証でもHTTPSを使えば暗号化されるので問題ない。ダイジェスト認証はHTTPSが使えない場合の認証方法。
pub(crate) fn build_nonce() -> Result<String> {
    let mut s = String::new();
    s.push_str(
        format!(
            "{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_nanos()
        )
        .as_str(),
    );
    s.push_str(format!("{}", rand::random::<u64>()).as_str());

    let mut h = Md5::new();
    h.update(s.as_bytes());
    Ok(format!("{:x}", h.finalize()))
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestType(u8);
pub const STUN_PACKET: RequestType = RequestType(0x00);
pub const CHANNEL_DATA: RequestType = RequestType(0x01);
pub const UNKNOWN_PACKET: RequestType = RequestType(0x77);
