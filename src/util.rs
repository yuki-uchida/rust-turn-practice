use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::UdpSocket;
pub type Result<T> = std::result::Result<T, Error>;
#[async_trait]
pub trait Conn {
    async fn connect(&self, addr: SocketAddr) -> Result<()>;
    async fn recv(&self, buf: &mut [u8]) -> Result<usize>;
    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)>;
    async fn send(&self, buf: &[u8]) -> Result<usize>;
    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize>;
    async fn local_addr(&self) -> Result<SocketAddr>;
    async fn remote_addr(&self) -> Option<SocketAddr>;
    async fn close(&self) -> Result<()>;
}

#[async_trait]
impl Conn for UdpSocket {
    async fn connect(&self, addr: SocketAddr) -> Result<()> {
        Ok(self.connect(addr).await?)
    }

    async fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.recv(buf).await?)
    }

    async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        Ok(self.recv_from(buf).await?)
    }

    async fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(self.send(buf).await?)
    }

    async fn send_to(&self, buf: &[u8], target: SocketAddr) -> Result<usize> {
        Ok(self.send_to(buf, target).await?)
    }

    async fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.local_addr()?)
    }

    async fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

// ここのエラーはTurnのエラーに集約する
#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    #[error("buffer: full")]
    ErrBufferFull,
    #[error("buffer: closed")]
    ErrBufferClosed,
    #[error("buffer: short")]
    ErrBufferShort,
    #[error("packet too big")]
    ErrPacketTooBig,
    #[error("i/o timeout")]
    ErrTimeout,
    #[error("udp: listener closed")]
    ErrClosedListener,
    #[error("udp: listen queue exceeded")]
    ErrListenQueueExceeded,
    #[error("udp: listener accept ch closed")]
    ErrClosedListenerAcceptCh,
    #[error("obs cannot be nil")]
    ErrObsCannotBeNil,
    #[error("se of closed network connection")]
    ErrUseClosedNetworkConn,
    #[error("addr is not a net.UDPAddr")]
    ErrAddrNotUdpAddr,
    #[error("something went wrong with locAddr")]
    ErrLocAddr,
    #[error("already closed")]
    ErrAlreadyClosed,
    #[error("no remAddr defined")]
    ErrNoRemAddr,
    #[error("address already in use")]
    ErrAddressAlreadyInUse,
    #[error("no such UDPConn")]
    ErrNoSuchUdpConn,
    #[error("cannot remove unspecified IP by the specified IP")]
    ErrCannotRemoveUnspecifiedIp,
    #[error("no address assigned")]
    ErrNoAddressAssigned,
    #[error("1:1 NAT requires more than one mapping")]
    ErrNatRequriesMapping,
    #[error("length mismtach between mappedIPs and localIPs")]
    ErrMismatchLengthIp,
    #[error("non-udp translation is not supported yet")]
    ErrNonUdpTranslationNotSupported,
    #[error("no associated local address")]
    ErrNoAssociatedLocalAddress,
    #[error("no NAT binding found")]
    ErrNoNatBindingFound,
    #[error("has no permission")]
    ErrHasNoPermission,
    #[error("host name must not be empty")]
    ErrHostnameEmpty,
    #[error("failed to parse IP address")]
    ErrFailedToParseIpaddr,
    #[error("no interface is available")]
    ErrNoInterface,
    #[error("not found")]
    ErrNotFound,
    #[error("unexpected network")]
    ErrUnexpectedNetwork,
    #[error("can't assign requested address")]
    ErrCantAssignRequestedAddr,
    #[error("unknown network")]
    ErrUnknownNetwork,
    #[error("no router linked")]
    ErrNoRouterLinked,
    #[error("invalid port number")]
    ErrInvalidPortNumber,
    #[error("unexpected type-switch failure")]
    ErrUnexpectedTypeSwitchFailure,
    #[error("bind failed")]
    ErrBindFailed,
    #[error("end port is less than the start")]
    ErrEndPortLessThanStart,
    #[error("port space exhausted")]
    ErrPortSpaceExhausted,
    #[error("vnet is not enabled")]
    ErrVnetDisabled,
    #[error("invalid local IP in static_ips")]
    ErrInvalidLocalIpInStaticIps,
    #[error("mapped in static_ips is beyond subnet")]
    ErrLocalIpBeyondStaticIpsSubset,
    #[error("all static_ips must have associated local IPs")]
    ErrLocalIpNoStaticsIpsAssociated,
    #[error("router already started")]
    ErrRouterAlreadyStarted,
    #[error("router already stopped")]
    ErrRouterAlreadyStopped,
    #[error("static IP is beyond subnet")]
    ErrStaticIpIsBeyondSubnet,
    #[error("address space exhausted")]
    ErrAddressSpaceExhausted,
    #[error("no IP address is assigned for eth0")]
    ErrNoIpaddrEth0,
    #[error("Invalid mask")]
    ErrInvalidMask,
    #[error("{0}")]
    Io(#[source] IoError),
    #[error("{0}")]
    Other(String),
}
#[derive(Debug, Error)]
#[error("io error: {0}")]
pub struct IoError(#[from] pub io::Error);
// Workaround for wanting PartialEq for io::Error.
impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(IoError(e))
    }
}
