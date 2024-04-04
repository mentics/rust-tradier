use std::env;
use futures_util::{StreamExt, SinkExt};
use serde_json::{Value,json};
use tokio::runtime::Builder;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

pub trait Handler<T> {
    fn on_data(&mut self, data:T);
}

pub fn start<H:Handler<String> + 'static + Send + Sync>(handler:H) {
    std::thread::spawn(move || {
        println!("Setting up separate thread for websocket client");
        let rt = Builder::new_current_thread().enable_io().enable_time().build().unwrap(); // new_multi_thread().worker_threads(4).enable_all().build().unwrap();
        // tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            run(handler).await;
        });
    });
}

async fn run<H:Handler<String> + 'static + Send + Sync>(mut handler:H) {
    println!("In websocket thread");
    // TODO: if stream breaks, try to fix it
    let (sid, ws_stream) = connect().await;
    let (mut write, mut read) = ws_stream.split();
    // let payload = format!("{{\"symbols\": [\"SPY\"], \"sessionid\": {sid}, \"linebreak\": false}}");
    let payload = json!({ "symbols": ["SPY"], "sessionid": sid, "linebreak": false }).to_string();
    println!("Payload sending: {}", payload);
    match write.send(Message::Text(payload)).await {
        Ok(o) => println!("Error when submitting subscription: {:?}", o),
        Err(err) => {
            println!("Error when submitting subscription: {:?}", err);
            return;
        },
    }
    loop {
        if let Some(msg) = read.next().await {
            println!("Received message: {:?}", msg);
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text: {:?}", text);
                    handler.on_data(text);
                }
                Ok(Message::Binary(bin)) => {
                    println!("Received binary: {:?}", bin);
                }
                Ok(Message::Ping(bin)) => {
                    println!("Received ping: {:?}", bin);
                }
                Ok(Message::Pong(bin)) => {
                    println!("Received pong: {:?}", bin);
                }
                Ok(Message::Close(msg)) => {
                    println!("Received close: {:?}", msg);
                    break;
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    break;
                },
                _ => {
                    println!("Other: {:?}", msg);
                    break;
                }
            }
        } else {
            println!("Error in websocket thread");
            break;
        }
    }
}

async fn connect() -> (String, WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) {
    let resp = tradier_post("/markets/events/session").await.unwrap();
    println!("{}", resp);
    let data = serde_json::from_str::<Value>(&resp).unwrap();
    let s = &data["stream"];
    let sid = s["sessionid"].as_str().unwrap().to_string();
    // let url = s["url"].as_str().unwrap();
    let url = "wss://ws.tradier.com/v1/markets/events";
    let url_parsed = reqwest::Url::parse(url).unwrap();
    println!("Connecting to websocket {} with session id {}", url, sid);

    let (ws_stream, _) = connect_async(url_parsed).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    return (sid, ws_stream);
}


use reqwest::Client;

async fn tradier_post(uri: &str) -> Result<String, reqwest::Error> {
    let api_key = env::var("TRADIER_API_KEY").unwrap();
    const BASE_URL: &str = "https://api.tradier.com/v1";
    let url = [BASE_URL, uri].concat();

    let client = Client::new();

    return client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        // .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Content-Length", 0) // body.len().to_string())
        .body("")
        .send()
        .await?
        .text()
        .await;

    // match response {
    //     Ok(res) => Ok(res),
    //     Err(e) => Err(e),
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test {
        data:String
    }

    impl Handler<String> for Test {
        fn on_data(&mut self, data:String) {
            println!("Handler::on_data called with {:?}", data);
            self.data = data;
        }
    }

    #[test]
    fn test_websocket() {
        let h = Test { data: "none yet".to_string() };
        start(h);
        std::thread::sleep(std::time::Duration::from_secs(4));
        println!("Test ending");
    }
}
