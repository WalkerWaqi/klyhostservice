mod config;
mod mqtt;

use log::info;
use log4rs;
use mqtt::*;
use std::error::Error;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    tokio::spawn(async {
        thread::sleep(Duration::from_secs(1));
        let _ = mqtt::Mqtt::publish_async("test", "111", 2);
        // thread::sleep(Duration::from_secs(1));
        let _ = mqtt::Mqtt::publish_block("test", "222", 2);
    });

    Mqtt::start().await?;

    Ok(())
}
