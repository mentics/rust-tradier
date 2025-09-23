#![allow(unused)]
use crate::http::tradier_post;
use chrono::{NaiveDateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream};

pub trait Handler<T> {
    fn on_data(&mut self, timestamp: NaiveDateTime, data: T);
}

// pub fn start<H:Handler<String> + 'static + Send + Sync>(mut handler:H, symbols:&str) {
//     let sym = symbol.to_string();
//     std::thread::spawn(move || {
//         println!("Setting up separate thread for websocket client");
//         let rt = Builder::new_current_thread().enable_io().enable_time().build().unwrap(); // new_multi_thread().worker_threads(4).enable_all().build().unwrap();
//         // tokio::runtime::Runtime::new().unwrap();
//         rt.block_on(async move {
//             while run(&mut handler, &sym).await {}
//         });
//     });
// }

// pub fn run_sync<H:Handler<String> + 'static + Send + Sync>(handler:H) {
//     println!("Setting up runtime for websocket client");
//     let rt = Builder::new_current_thread().enable_io().enable_time().build().unwrap(); // new_multi_thread().worker_threads(4).enable_all().build().unwrap();
//     // tokio::runtime::Runtime::new().unwrap();
//     rt.block_on(async move {
//         run(handler).await;
//     });
// }

/// symbols is comma separated string of symbols to subscribe
pub async fn run_async<H: Handler<String> + 'static + Send + Sync>(
    mut handler: H,
    symbols: &[&str]
) {
    println!("Setting up listening on websocket client");
    // let rt = Builder::new_current_thread().enable_io().enable_time().build().unwrap(); // new_multi_thread().worker_threads(4).enable_all().build().unwrap();
    // tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(async move {
    while run(&mut handler, symbols).await {}
    // });
}

/// Returns true if the caller should attempt to reconnect, or false if the caller should exit.
async fn run<H: Handler<String> + 'static + Send + Sync>(
    handler: &mut H,
    symbols: &[&str]
) -> bool {
    println!("In websocket thread");
    // TODO: if stream breaks, try to fix it
    let (sid, ws_stream) = connect().await;
    let (mut write, mut read) = ws_stream.split();
    // let symbols_str = symbols.join(",");
    let payload = json!({ "symbols": symbols, "sessionid": sid, "linebreak": false }).to_string();
    println!("Payload sending: {}", payload);
    match write.send(Message::Text(payload)).await {
        Ok(o) => println!("Successful subscription: {:?}", o),
        Err(err) => {
            println!("Error when submitting subscription: {:?}", err);
            return false;
        }
    }
    loop {
        match timeout(Duration::from_secs(100), read.next()).await {
            Err(elapsed) => {
                println!(
                    "{}: Websocket read timed out |{}|. Sending ping.",
                    Utc::now().naive_utc(),
                    elapsed
                );
                match write.send(Message::Ping(Vec::new())).await {
                    Ok(_) => continue,
                    Err(e) => {
                        println!("Exiting: Error sending ping after timeout. {}", e);
                        return false;
                    }
                }
            }

            Ok(None) => {
                println!("Exiting: Websocket read.next returned None.");
                return false;
            }

            Ok(Some(msg)) => {
                // if let Some(msg) = timeout(Duration::from_secs(100), read.next()).await {
                let now = Utc::now().naive_utc();
                // println!("Received message: {:?}", msg);
                match msg {
                    Ok(Message::Text(payload)) => {
                        // println!("Received text: {:?}", text);
                        handler.on_data(now, payload);
                    }
                    Ok(Message::Binary(payload)) => {
                        println!("{}: Received binary: {:?}", now, payload);
                    }
                    Ok(Message::Ping(payload)) => {
                        println!("{}: Received ping: {:?}", now, payload);
                    }
                    Ok(Message::Pong(payload)) => {
                        println!("{}: Received pong: {:?}", now, payload);
                    }
                    Ok(Message::Close(payload)) => {
                        println!("{}: Exiting: Received close: {:?}", now, payload);
                        return false;
                    }
                    Err(e) => {
                        println!("Error at {:?}: {:?}", now, e);
                        break;
                    }
                    _ => {
                        println!("Other at {:?}: {:?}", now, msg);
                        break;
                    }
                }
            }
        }
    }
    true
}

async fn connect() -> (
    String,
    WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>
) {
    let resp = tradier_post("/markets/events/session", None).await.unwrap();
    println!("{}", resp);
    let data = serde_json::from_str::<Value>(&resp).unwrap();
    let s = &data["stream"];
    let sid = s["sessionid"].as_str().unwrap().to_string();
    // let url = s["url"].as_str().unwrap();
    // See: https://documentation.tradier.com/brokerage-api/streaming/get-markets-events
    let url = "wss://ws.tradier.com/v1/markets/events";
    let url_parsed = reqwest::Url::parse(url).unwrap();
    println!("Connecting to websocket {} with session id {}", url, sid);

    let (ws_stream, _) = connect_async(url_parsed).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    (sid, ws_stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::arch::asm;

    #[tokio::test]
    async fn test_run_async() {
        // let h = Test { data: "none yet".to_string() };
        // run_sync(h);
        struct HH(u16);
        impl Handler<String> for HH {
            fn on_data(&mut self, _timestamp: NaiveDateTime, data: String) {
                println!("Handler::on_data called, msg received {:?}", data);
                self.0 += 1;
                if self.0 > 2 {
                    println!("Test run_sync ending");
                    std::process::exit(0);
                }
            }
        }
        run_async(HH(0), &["SPY"]).await;
        std::thread::sleep(std::time::Duration::from_secs(4));
        println!("Test run_async ending");
    }

    #[test]
    fn test_timing() {
        unsafe {
            let t1 = core::arch::x86_64::_rdtsc();
            let t2 = core::arch::x86_64::_rdtsc();
            println!("Without asm elapsed {}", t2 - t1);
        }
    }

    #[test]

    fn test_timing_asm_separate() {
        unsafe {
            let mut t1low: u32 = 0;
            let mut t1high: u32 = 0;
            let mut t2low: u32 = 0;
            let mut t2high: u32 = 0;
            let mut t3low: u32 = 0;
            let mut t3high: u32 = 0;

            asm!(
                "rdtsc",
                out("eax") t1low,
                out("edx") t1high,
                options(nostack, pure, nomem)
            );

            asm!(
                "rdtsc",
                out("eax") t2low,
                out("edx") t2high,
                options(nostack, pure, nomem)
            );

            asm!(
                "rdtsc",
                out("eax") t3low,
                out("edx") t3high,
                options(nostack, pure, nomem)
            );

            let t1 = (t1high as u64) << 32 | t1low as u64;
            let t2 = (t2high as u64) << 32 | t2low as u64;
            let t3 = (t3high as u64) << 32 | t3low as u64;
            println!(
                "Asm separate elapsed 1-2: {}, elapsed 2-3: {}, elapsed 1-3: {},",
                (t2 - t1),
                (t3 - t2),
                (t3 - t1)
            );
        }
    }

    #[test]
    fn test_timing_asm_combined() {
        unsafe {
            let mut t1low: u32;
            let mut t1high: u32;
            let mut t2low: u32;
            let mut t2high: u32;

            asm!(
                "rdtsc",
                "mov r8d, eax",
                "mov r9d, edx",
                "rdtsc",
                out("r8d") t1low,
                out("r9d") t1high,
                out("eax") t2low,
                out("edx") t2high,
                options(nostack, pure, nomem)
            );

            let t1 = ((t1high as u64) << 32) | t1low as u64;
            let t2 = ((t2high as u64) << 32) | t2low as u64;
            println!("Asm combined elapsed {}", t2 - t1);

            println!("time: {}", Utc::now().naive_utc());
        }
    }
}
