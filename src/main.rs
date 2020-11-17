mod config;
mod mqtt;

use futures::executor::block_on;
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
        thread::sleep(Duration::from_secs(3));

        let _: Result<(), Box<dyn Error>> = block_on(async {
            let mqtt = Mqtt::get_instance();
            let mqtt = mqtt.lock().unwrap();

            mqtt.publish("test", "hello", 2).await?;

            Ok(())
        });
    });

    Mqtt::start().await?;

    Ok(())
}
