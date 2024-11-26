pub mod sync;
pub mod test;
use std::time::Duration;

use rumqttc::v5::{mqttbytes::QoS, AsyncClient, EventLoop, MqttOptions};
use tokio::time;

// pub struct MqttConnection {
//     pub client: MqttClient,
//     pub eventloop: EventLoop,
// }

pub struct MqttAsyncClient {
    client: AsyncClient,
}

impl MqttAsyncClient {
    // pub async fn new()
    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }

    pub async fn publish(&self, topic: &str, qos: QoS, payload: &str) {
        self.client
            .publish(topic, qos, false, payload.as_bytes().to_vec())
            .await
            .unwrap();
    }
}
pub struct MqttClientBuilder {
    client_id: String,
    url: String,
    username: String,
    password: String,
    user_property: Vec<(String, String)>,
}

impl MqttClientBuilder {
    pub fn new(
        client_id: &str,
        url: &str,
        username: &str,
        password: &str,
        user_property: Vec<(String, String)>,
    ) -> Self {
        Self {
            client_id: client_id.to_string(),
            url: url.to_string(),
            user_property,
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn build(self) -> Result<(MqttAsyncClient, EventLoop), crate::errors::TransportError> {
        let url = format!("{}?client_id={}", self.url, self.client_id);
        // let url = format!("{}?client_id=1111", self.url);
        let mut mqttoptions = MqttOptions::parse_url(url)?;

        // let mut mqttoptions = MqttOptions::new(self.client_id, self.url, 1883);
        // let mut connect_props = rumqttc::v5::mqttbytes::v5::ConnectProperties::new();
        // connect_props.session_expiry_interval = Some(2 * 60 * 60);
        // mqttoptions.set_credentials(self.username, self.password);
        // mqttoptions.set_connect_properties(connect_props);
        // mqttoptions.set_keep_alive(Duration::from_secs(10));
        // mqttoptions.set_transport(rumqttc::Transport::Tcp);
        // mqttoptions
        //     .set_user_properties(self.user_property)
        //     .set_connection_timeout(20)
        //     .set_clean_start(false)
        //     .set_manual_acks(true);

        mqttoptions
            // .set_connect_properties(connect_props)
            .set_transport(rumqttc::Transport::Tcp)
            .set_connection_timeout(20)
            .set_credentials(self.username, self.password)
            .set_keep_alive(Duration::from_secs(10))
            .set_user_properties(self.user_property)
            .set_clean_start(true)
            .set_manual_acks(true);

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

        Ok((MqttAsyncClient { client }, eventloop))
    }
}

// 测试用
pub async fn _mqtt_connect(username: &str, password: &str, content: &str, client_id: &str) {
    let user_property = vec![
        ("content".to_string(), content.to_string()),
        ("clientId".to_string(), client_id.to_string()),
    ];

    let url = format!("{}?client_id={}", "mqtt://126.214.108.58:11883", client_id);
    // let mut mqttoptions = MqttOptions::new(client_id, "ws://100.106.144.126:8083/mqtt", 8000);
    let mut mqttoptions = MqttOptions::parse_url(url).unwrap();
    // let mut mqttoptions = MqttOptions::new(client_id, "126.214.108.58", 11883);
    let mut connect_props = rumqttc::v5::mqttbytes::v5::ConnectProperties::new();
    connect_props.session_expiry_interval = Some(2 * 60 * 60); // 2 days in seconds

    mqttoptions
        .set_connect_properties(connect_props)
        .set_transport(rumqttc::Transport::Tcp)
        .set_credentials(username, password)
        .set_keep_alive(Duration::from_secs(10))
        .set_user_properties(user_property)
        .set_clean_start(false)
        .set_manual_acks(true);

    let (_client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    while let Ok(event) = eventloop.poll().await {
        tracing::info!("{event:?}");

        if let rumqttc::v5::Event::Incoming(packet) = event {
            let _publish = match packet {
                rumqttc::v5::mqttbytes::v5::Packet::Publish(publish) => publish,
                _ => continue,
            };
            // this time we will ack incoming publishes.
            // Its important not to block notifier as this can cause deadlock.
            // let c = client.clone();
            // tokio::spawn(async move {
            //     c.ack(&publish).await.unwrap();
            // });
        }
    }
}

async fn _requests(client: AsyncClient) {
    client
        .subscribe("hello/world", QoS::AtMostOnce)
        .await
        .unwrap();

    for i in 1..=10 {
        client
            .publish("hello/world", QoS::ExactlyOnce, false, vec![1; i])
            .await
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }

    time::sleep(Duration::from_secs(120)).await;
}
#[cfg(test)]
mod tests {
    use wallet_utils::init_log;

    use super::*;
    #[tokio::test]
    async fn test_mqtt() {
        init_log();
        // let client_id = "wenjing";
        // let device_type = "ANDROID";
        // let sn = "guangxiang";
        // let app_id = "xxxxxxx";

        let device_type = "ANDROID";
        let sn = "104.2.0.125C00";
        let username = "104.2.0.125C00";
        let password = "ada7d9308190fe45";
        let identifier = format!("{}_{}_{}", password, sn, device_type);
        let client_id = wallet_utils::md5(&identifier);

        let content = wallet_utils::ecb::Aes128EcbCryptor::new(password)
            .unwrap()
            .encrypt(&identifier)
            .unwrap();
        _mqtt_connect(username, password, &content, &client_id).await
    }

    // #[tokio::test]
    // async fn test_init_mqtt() {
    //     let url = "ws://100.106.144.126:8083/mqtt";
    //     let device_type = "ANDROID";
    //     let sn = "wenjing";
    //     let client_id = "wenjing";
    //     let app_id = "666";
    //     init_mqtt_processor(url, device_type, sn, client_id, app_id, handle_eventloop);
    // }
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Claims {
        iss: String,
        sub: String,
        // 其他需要的声明
    }
}
