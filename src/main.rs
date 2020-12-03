mod config;
mod mqtt;
mod protos;
mod sysvirt;
mod unixsocket;

use log::info;
use log4rs;
use mqtt::Mqtt;
use std::error::Error;
use sysvirt::Virt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    let virt = Virt::new("qemu:///system");
    virt.start();

    Mqtt::start().await?;

    Ok(())
}
