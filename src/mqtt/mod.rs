use super::config;
use futures::{stream::StreamExt, Stream};
use log::{error, info};
use paho_mqtt as mqtt;
use std::{error::Error, pin::Pin, process, time::Duration};

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[mqtt::QOS_2, mqtt::QOS_2];

struct Mqtt {
    cli: mqtt::AsyncClient,
    strm: Pin<Box<dyn Stream<Item = Option<mqtt::Message>>>>,
}

impl Mqtt {
    fn new(url: &str) -> Mqtt {
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(url)
            .client_id("klyhostservice_subscribe")
            .finalize();

        let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
            println!("Error creating the client: {:?}", err);
            process::exit(1);
        });

        Mqtt {
            strm: Box::pin(cli.get_stream(25)),
            cli,
        }
    }

    async fn connect(&mut self, will_topic: &str, will_payload: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        let lwt = mqtt::Message::new(will_topic, will_payload, qos);
        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(false)
            .will_message(lwt)
            .finalize();

        self.cli.connect(conn_opts).await?;

        Ok(())
    }

    async fn subscribe(&mut self, topic: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        self.cli.subscribe(topic, qos).await?;

        Ok(())
    }

    async fn subscribe_many(&mut self, topics: &[&str], qos: &[i32]) -> Result<(), Box<dyn Error>> {
        self.cli.subscribe_many(topics, qos).await?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(msg_opt) = self.strm.next().await {
            if let Some(msg) = msg_opt {
                info!("{}", msg);
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                info!("Lost connection. Attempting reconnect.");
                while let Err(err) = self.cli.reconnect().await {
                    error!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        Ok(())
    }
}

pub async fn start() -> Result<(), Box<dyn Error>> {
    let mut mqtt = Mqtt::new(&config::CONFIG.mqtt.url);

    info!("Connecting to the MQTT server...");
    mqtt.connect("test", "Async subscriber lost connection", mqtt::QOS_2).await?;

    info!("Subscribing to topics: {:?}", TOPICS);
    mqtt.subscribe_many(TOPICS, QOS).await?;
    mqtt.subscribe("abc", mqtt::QOS_2).await?;

    info!("Waiting for messages...");
    mqtt.receive().await?;

    Ok(())
}
