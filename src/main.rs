mod config;
mod mqtt;

use log::info;
use log4rs;
use mqtt::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    Mqtt::start().await?;

    Ok(())
}
