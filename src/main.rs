mod config;
mod mqtt;

use futures::executor::block_on;
use log::{error, info};
use log4rs;
use mqtt::*;
use paho_mqtt;
use std::error::Error;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    info!("---------- klyhostservice started ----------");
    info!("{:#?}", *config::CONFIG);

    tokio::spawn(async {
        thread::sleep(Duration::from_secs(3));

        if let Err(err) = block_on(async {
            let mqtt = Mqtt::get_instance();
            let mqtt = mqtt.lock().unwrap();

            let msg = paho_mqtt::Message::new("test", "hello", 2);
            mqtt.cli.publish(msg).await?;

            Ok::<(), paho_mqtt::Error>(())
        }) {
            error!("{}", err);
        }
    });

    Mqtt::start().await?;

    Ok(())
}
