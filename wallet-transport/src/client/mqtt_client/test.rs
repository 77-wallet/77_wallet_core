use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use tokio::time;

use rumqttc::v5::{AsyncClient, Event, EventLoop, MqttOptions};
use std::error::Error;
use std::time::Duration;

fn create_conn(
    username: &str,
    password: &str,
    content: &str,
    client_id: &str,
) -> (AsyncClient, EventLoop) {
    let user_property = vec![
        ("content".to_string(), content.to_string()),
        ("clientId".to_string(), client_id.to_string()),
    ];

    let url = format!("{}?client_id={}", "mqtt://126.214.108.58:11883", client_id);
    // let mut mqttoptions = MqttOptions::new(client_id, "mqtt://126.214.108.58", 11883);
    let mut mqttoptions = MqttOptions::parse_url(url).unwrap();
    let mut connect_props = rumqttc::v5::mqttbytes::v5::ConnectProperties::new();
    connect_props.session_expiry_interval = Some(u32::MAX); // 2 days in seconds

    mqttoptions
        .set_connect_properties(connect_props)
        .set_credentials(username, password)
        .set_keep_alive(Duration::from_secs(5))
        .set_user_properties(user_property)
        .set_manual_acks(true)
        .set_clean_start(false);

    AsyncClient::new(mqttoptions, 10)
}

#[allow(dead_code)]
async fn test(
    username: &str,
    password: &str,
    content: &str,
    client_id: &str,
) -> Result<(), Box<dyn Error>> {
    // create mqtt connection with clean_session = false and manual_acks = true
    let (client, mut eventloop) = create_conn(username, password, content, client_id);

    // subscribe example topic
    client
        .subscribe("hello/world", QoS::AtLeastOnce)
        .await
        .unwrap();

    let _client_c = client.clone();
    // task::spawn(async move {
    //     // send some messages to example topic and disconnect
    //     requests(&client_c).await;
    //     client_c.disconnect().await.unwrap()
    // });

    // get subscribed messages without acking
    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(v) => {
                tracing::info!("Event = {v:?}");
                match v {
                    Event::Incoming(packet) => {
                        let publish = match packet {
                            Packet::Publish(publish) => publish,
                            _ => continue,
                        };

                        // this time we will ack incoming publishes.
                        // Its important not to block notifier as this can cause deadlock.
                        let c = client.clone();
                        tokio::spawn(async move {
                            c.ack(&publish).await.unwrap();
                        });
                    }
                    Event::Outgoing(outgoing) => {
                        tracing::info!("Outgoing: {:?}", outgoing);
                    }
                }
            }
            Err(e) => {
                tracing::info!("Error = {e:?}");
                break;
            }
        }
    }

    Ok(())
}
#[allow(dead_code)]
async fn requests(client: &AsyncClient) {
    for i in 1..=10 {
        client
            .publish("hello/world", QoS::AtLeastOnce, false, vec![1; i])
            .await
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use wallet_utils::init_log;

    use super::*;
    #[tokio::test]
    async fn test_mqtt() {
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

        let _ = test(username, password, &content, &client_id).await;
    }
}
