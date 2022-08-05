use stun::attribute::*;
use stun::error::*;
use stun::message::*;
// 特定のプロトコルを使用するためにクライアントから送信される
pub struct RequestedTransport {
    pub protocol: Protocol,
}

const REQUESTED_TRANSPORT_SIZE: usize = 4;

impl Setter for RequestedTransport {
    fn set_extra_attribute(&self, m: &mut Message) -> Result<()> {
        let (mut raw, length) = (
            Vec::with_capacity(REQUESTED_TRANSPORT_SIZE),
            REQUESTED_TRANSPORT_SIZE,
        );
        // extra_attribute
        raw.extend_from_slice(&[0; REQUESTED_TRANSPORT_SIZE]);
        raw[0] = self.protocol.0;
        let extra_attribute = Attribute::new(
            ATTR_REQUESTED_TRANSPORT,
            REQUESTED_TRANSPORT_SIZE as u16,
            raw,
        );
        m.attributes.push(extra_attribute);
        Ok(())
    }
}

pub struct Protocol(pub u8);
pub const PROTO_TCP: Protocol = Protocol(6);
pub const PROTO_UDP: Protocol = Protocol(17);
