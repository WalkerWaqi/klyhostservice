use super::config;
use futures::{executor::block_on, stream::StreamExt, Stream};
use log::{error, info};
use paho_mqtt as mqtt;
use std::sync::Arc;
use std::{error::Error, pin::Pin, process, time::Duration};
use tokio::sync::Mutex;

const TOPICS: &[&str] = &["test", "hello"];
const QOS: &[i32] = &[mqtt::QOS_2, mqtt::QOS_2];

pub struct Mqtt {
    pub cli: mqtt::AsyncClient,
}

#[allow(dead_code)]
impl Mqtt {
    fn new(url: &str) -> Mqtt {
        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(url)
            .client_id("klyhostservice_subscribe")
            .finalize();

        Mqtt {
            cli: mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
                println!("Error creating the client: {:?}", err);
                process::exit(1);
            }),
        }
    }

    async fn connect(
        &self,
        will_topic: &str,
        will_payload: &str,
        qos: i32,
    ) -> Result<(), Box<dyn Error>> {
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

    async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        self.cli.subscribe(topic, qos).await?;

        Ok(())
    }

    async fn subscribe_many(&self, topics: &[&str], qos: &[i32]) -> Result<(), Box<dyn Error>> {
        self.cli.subscribe_many(topics, qos).await?;

        Ok(())
    }

    async fn unsubscribe(&self, topic: &str) -> Result<(), Box<dyn Error>> {
        self.cli.unsubscribe(topic).await?;

        Ok(())
    }

    async fn unsubscribe_many(&self, topics: &[&str]) -> Result<(), Box<dyn Error>> {
        self.cli.unsubscribe_many(topics).await?;

        Ok(())
    }

    pub async fn publish(
        &self,
        topic: &str,
        payload: &str,
        qos: i32,
    ) -> Result<(), Box<dyn Error>> {
        let msg = paho_mqtt::Message::new(topic, payload, qos);
        self.cli.publish(msg).await?;

        Ok(())
    }

    pub async fn publish_async(topic: &str, payload: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        let mqtt = Mqtt::get_instance();
        let mqtt = mqtt.lock().await;
        mqtt.publish(topic, payload, qos).await?;
        Ok(())
    }

    pub fn publish_nonblock(topic: &str, payload: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        let topic = topic.to_string();
        let payload = payload.to_string();
        let qos = qos;
        tokio::spawn(async move {
            let _: Result<(), Box<dyn Error>> = block_on(async {
                let mqtt = Mqtt::get_instance();
                let mqtt = mqtt.lock().await;
                mqtt.publish(topic.as_ref(), payload.as_ref(), qos).await?;
                Ok(())
            });
        });

        Ok(())
    }

    pub fn publish_block(topic: &str, payload: &str, qos: i32) -> Result<(), Box<dyn Error>> {
        block_on(async {
            let mqtt = Mqtt::get_instance();
            let mqtt = mqtt.lock().await;

            mqtt.publish(topic, payload, qos).await?;

            Ok(())
        })
    }

    pub fn get_instance() -> Arc<Mutex<Mqtt>> {
        static mut POINT: Option<Arc<Mutex<Mqtt>>> = None;

        unsafe {
            POINT
                .get_or_insert_with(|| Arc::new(Mutex::new(Mqtt::new(&config::CONFIG.mqtt.url))))
                .clone()
        }
    }

    pub async fn start() -> Result<(), Box<dyn Error>> {
        let mut strm: Pin<Box<dyn Stream<Item = Option<mqtt::Message>>>>;
        {
            let mqtt = Mqtt::get_instance();
            let mut mqtt = mqtt.lock().await;

            info!("Connecting to the MQTT server...");
            mqtt.connect("test", "Async subscriber lost connection", mqtt::QOS_2)
                .await?;

            info!("Subscribing to topics: {:?}", TOPICS);
            mqtt.subscribe_many(TOPICS, QOS).await?;

            info!("Waiting for messages...");
            strm = Box::pin(mqtt.cli.get_stream(25));
        }
        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                info!("{}", msg);
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                info!("Lost connection. Attempting reconnect.");
                let mqtt = Mqtt::get_instance();
                let mqtt = mqtt.lock().await;
                while let Err(err) = mqtt.cli.reconnect().await {
                    error!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn notihing() {
        tokio::spawn(async {
            let _ = super::Mqtt::publish_nonblock("test", "111", 2);
            // thread::sleep(Duration::from_secs(1));
            let _ = super::Mqtt::publish_block("test", "222", 2);
        });
    }
}
