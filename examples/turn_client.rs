use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use turn::client::*;
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let host = "127.0.0.1";
    let port = "3478";
    // let credential = user.splitn(2, "=").collect();
    let connection = Arc::new(UdpSocket::bind("0.0.0.0:3301").await?);
    // clientの初期化
    let config = ClientConfig {
        turn_server_address: "127.0.0.1:3479".to_string(),
        username: "user".to_string(),
        password: "password".to_string(),
        connection: connection,
    };
    let client = Client::new(config).await?;
    client.listen().await?;

    // allocateリクエストを出し、TURN側のアドレス(IP+Port)がどこかを受け取る
    let relay_conn = client.allocate().await?;
    // pingが届くか確認する(10packetくらい送る)
    // if ping {
    //     do_ping_test(&client, relay_conn).await?;
    // }

    // 終了
    Ok(())
}
