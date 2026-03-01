use futures::StreamExt;
use iced::Subscription;

use crate::model::ServerMessage;

#[derive(Debug, Clone)]
pub enum WsEvent {
    Connected,
    Disconnected,
    MessageReceived(ServerMessage),
    Error(String),
}

pub fn connect(url: String) -> Subscription<WsEvent> {
    Subscription::run_with(url, create_ws_stream)
}

fn create_ws_stream(url: &String) -> impl futures::Stream<Item = WsEvent> {
    let url = url.clone();
    iced::stream::channel(100, async move |mut output: futures::channel::mpsc::Sender<WsEvent>| {
        use futures::SinkExt;
        use tokio_tungstenite::tungstenite::Message as WsMessage;

        loop {
            match tokio_tungstenite::connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    let _ = output.send(WsEvent::Connected).await;
                    let (_, mut read) = ws_stream.split();

                    loop {
                        match read.next().await {
                            Some(Ok(WsMessage::Text(text))) => {
                                match serde_json::from_str::<ServerMessage>(&text) {
                                    Ok(msg) => {
                                        let _ =
                                            output.send(WsEvent::MessageReceived(msg)).await;
                                    }
                                    Err(e) => {
                                        let _ = output
                                            .send(WsEvent::Error(format!("Parse: {}", e)))
                                            .await;
                                    }
                                }
                            }
                            Some(Ok(WsMessage::Close(_))) | None => {
                                let _ = output.send(WsEvent::Disconnected).await;
                                break;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                let _ = output
                                    .send(WsEvent::Error(format!("WS: {}", e)))
                                    .await;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = output
                        .send(WsEvent::Error(format!("Connect: {}", e)))
                        .await;
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}
