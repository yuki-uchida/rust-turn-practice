use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

use crate::request::Request;
use crate::util::*;
pub const INBOUND_MTU: usize = 1500;

pub struct Server {
    shutdown_tx: Mutex<Option<watch::Sender<bool>>>,
    realm: String,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        config.validate()?;
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let s = Server {
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
            realm: String::from("ucchy-webrtc-realm"),
        };
        tokio::spawn(async move {
            let _ = Server::read_loop(config.conn_config, shutdown_rx).await;
        });

        Ok(s)
    }

    async fn read_loop(conn: Arc<dyn Conn + Send + Sync>, mut shutdown_rx: watch::Receiver<bool>) {
        let mut buf = vec![0u8; INBOUND_MTU];
        loop {
            let (n, addr) = tokio::select! {
                v = conn.recv_from(&mut buf) => {
                    match v {
                        Ok(v) => v,
                        Err(err) => {
                            log::debug!("exit read loop on error: {}", err);
                            break;
                        }
                    }
                },
                did_change = shutdown_rx.changed() => {
                    if did_change.is_err() || *shutdown_rx.borrow() {
                        // if did_change.is_err, sender was dropped, or if
                        // bool is set to true, that means we're shutting down.
                        break
                    } else {
                        continue;
                    }
                }
            };
            println!("{:?}", &buf[..n]);
            let mut request = Request::new(buf[..n].to_vec(), addr).expect("Cant decode packet");
            println!("{:?}", request);
            if let Err(err) = request.handle_request().await {
                log::error!("error when handling datagram: {}", err);
            }
        }
    }

    pub async fn close(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(tx) = shutdown_tx.take() {
            // errors if there are no receivers, but that's irrelevant.
            let _ = tx.send(true);
            // wait for all receivers to drop/close.
            let _ = tx.closed().await;
        }

        Ok(())
    }
}

pub struct ServerConfig {
    pub conn_config: Arc<dyn Conn + Send + Sync>,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
