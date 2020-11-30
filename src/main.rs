mod config;
mod mqtt;
mod unixsocket;

// use futures::executor::block_on;
use log::info;
use log4rs;
use mqtt::Mqtt;
use std::error::Error;
// use unixsocket::UnixSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    // let a = UnixSocket::new("/tmp/uuu.sock");
    //         a.run().await?;

    Mqtt::start().await?;

    Ok(())
}
