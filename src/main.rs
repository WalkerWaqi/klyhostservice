mod config;
mod mqtt;
mod unixsocket;

use log::info;
use log4rs;
use mqtt::Mqtt;
use std::error::Error;
use unixsocket::UnixSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    let _ = UnixSocket::get_instance()
        .lock()
        .await
        .run("/tmp/aaa.sock".to_string())
        .await?;
    let _ = UnixSocket::get_instance()
        .lock()
        .await
        .run("/tmp/bbb.sock".to_string())
        .await?;

    Mqtt::start().await?;

    Ok(())
}
