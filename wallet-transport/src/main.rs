use rumqttc::{AsyncClient, MqttOptions, QoS, Transport};
use std::{error::Error, time::Duration};
use tokio::{task, time};

pub mod client;
pub mod errors;
pub mod request_builder;
pub mod types;

pub use errors::TransportError;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let device_type = "ANDROID";
    let sn = "wenjing";
    let client_id = "wenjing";
    let app_id = "666";
    client::mqtt_client::_mqtt_connect(device_type, sn, client_id, app_id).await;
    Ok(())
}

async fn _test() -> Result<(), Box<dyn Error>> {
    // port parameter is ignored when scheme is websocket
    let mut mqttoptions = MqttOptions::new("wenjing", "ws://100.106.144.126:8083/mqtt", 8083);

    mqttoptions.set_transport(Transport::Ws);
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    task::spawn(async move {
        _requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(notif) => {
                println!("Event = {notif:?}");
            }
            Err(err) => {
                println!("Error = {err:?}");
                return Ok(());
            }
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
