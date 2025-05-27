use std::time::Duration;

use rumqttc::v5::{Client, Connection, MqttOptions, mqttbytes::QoS};
use tokio::time;

pub struct MqttClient {
    client: Client,
}

impl MqttClient {
    // pub async fn new()
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    pub fn publish(&self, topic: &str, qos: QoS, payload: &str) {
        self.client
            .publish(topic, qos, false, payload.as_bytes().to_vec())
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

    pub fn build(self) -> Result<(MqttClient, Connection), crate::errors::TransportError> {
        let url = format!("{}?client_id={}", self.url, self.client_id);
        // let url = format!("{}?client_id=1111", self.url);
        let mut mqttoptions = MqttOptions::parse_url(url)?;

        // let mut mqttoptions = MqttOptions::new(self.client_id, self.url, 1883);
        let mut connect_props = rumqttc::v5::mqttbytes::v5::ConnectProperties::new();
        connect_props.session_expiry_interval = Some(2 * 60 * 60); // 2 days in seconds
        mqttoptions.set_credentials(self.username, self.password);
        mqttoptions.set_connect_properties(connect_props);
        mqttoptions.set_keep_alive(Duration::from_secs(10));
        mqttoptions.set_transport(rumqttc::Transport::Tcp);
        mqttoptions
            .set_user_properties(self.user_property)
            .set_connection_timeout(20)
            .set_clean_start(false)
            .set_manual_acks(true);

        let (client, eventloop) = Client::new(mqttoptions, 10);

        Ok((MqttClient { client }, eventloop))
    }
}

// 测试用
pub fn _mqtt_connect(username: &str, password: &str, content: &str, client_id: &str) {
    let user_property = vec![
        ("content".to_string(), content.to_string()),
        ("clientId".to_string(), client_id.to_string()),
    ];

    let url = format!("{}?client_id={}", "mqtt://126.214.108.58:11883", client_id);
    // let url = format!("{}?client_id=1111", self.url);
    let mut mqttoptions = MqttOptions::parse_url(url).unwrap();
    // let mut mqttoptions = MqttOptions::new(client_id, "126.214.108.58", 11883);
    let connect_props = rumqttc::v5::mqttbytes::v5::ConnectProperties::new();
    // connect_props.session_expiry_interval = Some(2 * 60 * 60); // 2 days in seconds
    mqttoptions.set_credentials(username, password);
    mqttoptions.set_connect_properties(connect_props);
    mqttoptions.set_keep_alive(Duration::from_secs(10));
    mqttoptions.set_manual_acks(true);
    mqttoptions.set_transport(rumqttc::Transport::Tcp);
    mqttoptions.set_user_properties(user_property);
    mqttoptions.set_connection_timeout(20);
    mqttoptions.set_clean_start(false);

    let (_client, mut eventloop) = Client::new(mqttoptions, 10);

    // while let Ok(notification) = eventloop.recv() {
    //     match notification {
    //         Ok(notif) => {
    //             println!("Event = {notif:?}");
    //         }
    //         Err(err) => {
    //             println!("Error = {err:?}");
    //         }
    //     }
    // }
    for notification in eventloop.iter() {
        match notification {
            Ok(notif) => {
                tracing::info!("Event = {notif:?}");
            }
            Err(err) => {
                tracing::info!("Error = {err:?}");
            }
        }
    }
}

async fn _requests(client: Client) {
    client.subscribe("hello/world", QoS::AtMostOnce).unwrap();

    for i in 1..=10 {
        client
            .publish("hello/world", QoS::ExactlyOnce, false, vec![1; i])
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }

    time::sleep(Duration::from_secs(120)).await;
}
#[cfg(test)]
mod tests {
    use wallet_utils::init_log;

    use super::*;

    #[test]
    fn test_mqtt() {
        init_log();
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
        _mqtt_connect(username, password, &content, &client_id)
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
