use super::config;
use futures::stream::StreamExt;
use log::{error, info};
use paho_mqtt as mqtt;
use std::{error::Error, process, time::Duration};

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[mqtt::QOS_2, mqtt::QOS_2];

pub async fn start() -> Result<(), Box<dyn Error>> {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(&config::CONFIG.mqtt.url)
        .client_id("klyhostservice_subscribe")
        .finalize();

    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let mut strm = cli.get_stream(25);
    let lwt = mqtt::Message::new("test", "Async subscriber lost connection", mqtt::QOS_2);
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .keep_alive_interval(Duration::from_secs(20))
        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
        .clean_session(false)
        .will_message(lwt)
        .finalize();

    info!("Connecting to the MQTT server...");
    cli.connect(conn_opts).await?;

    info!("Subscribing to topics: {:?}", TOPICS);
    cli.subscribe_many(TOPICS, QOS).await?;

    info!("Waiting for messages...");

    while let Some(msg_opt) = strm.next().await {
        if let Some(msg) = msg_opt {
            info!("{}", msg);
        } else {
            // A "None" means we were disconnected. Try to reconnect...
            info!("Lost connection. Attempting reconnect.");
            while let Err(err) = cli.reconnect().await {
                error!("Error reconnecting: {}", err);
                // For tokio use: tokio::time::delay_for()
                async_std::task::sleep(Duration::from_millis(1000)).await;
            }
        }
    }

    Ok(())
}
