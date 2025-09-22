use rust_tradier::LiveDataSubscriptionManager;
use tokio::select;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Starting Live Data Subscription Manager Example");

    // Create a cancellation token for clean shutdown
    let cancellation_token = CancellationToken::new();

    // Create the subscription manager
    let manager = LiveDataSubscriptionManager::<String>::new();

    // Create channels for different clients
    let (client1_tx, mut client1_rx) = mpsc::channel::<rust_tradier::MarketData<String>>(100);
    let (client2_tx, mut client2_rx) = mpsc::channel::<rust_tradier::MarketData<String>>(100);

    // Clone cancellation token for client tasks
    let client1_cancel = cancellation_token.clone();

    // Spawn tasks to handle incoming data for each client
    let client1_handle = tokio::spawn(async move {
        println!("Client1 data receiver started");
        loop {
            select! {
                _ = client1_cancel.cancelled() => {
                    println!("Client1 data receiver cancelled");
                    break;
                }
                market_data = client1_rx.recv() => {
                    match market_data {
                        Some(data) => {
                            println!(
                                "[Client1] {} - {}: {}",
                                data.timestamp, data.symbol, data.data
                            );
                        }
                        None => {
                            println!("Client1 channel closed");
                            break;
                        }
                    }
                }
            }
        }
        println!("Client1 data receiver ended");
    });

    let client2_cancel = cancellation_token.clone();

    let client2_handle = tokio::spawn(async move {
        println!("Client2 data receiver started");
        loop {
            select! {
                _ = client2_cancel.cancelled() => {
                    println!("Client2 data receiver cancelled");
                    break;
                }
                market_data = client2_rx.recv() => {
                    match market_data {
                        Some(data) => {
                            println!(
                                "[Client2] {} - {}: {}",
                                data.timestamp, data.symbol, data.data
                            );
                        }
                        None => {
                            println!("Client2 channel closed");
                            break;
                        }
                    }
                }
            }
        }
        println!("Client2 data receiver ended");
    });

    // Client 1 subscribes to some tech stocks
    println!("\nClient 1 subscribing to AAPL and MSFT");
    manager
        .subscribe(
            "client1".to_string(),
            vec!["AAPL".to_string(), "MSFT".to_string()],
            client1_tx.clone()
        )
        .await
        .unwrap();

    // Client 2 subscribes to different stocks
    println!("Client 2 subscribing to GOOGL and TSLA");
    manager
        .subscribe(
            "client2".to_string(),
            vec!["GOOGL".to_string(), "TSLA".to_string()],
            client2_tx.clone()
        )
        .await
        .unwrap();

    // Wait a moment for subscriptions to be processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check current subscriptions
    let client1_subs = manager.get_client_subscriptions("client1").await;
    let client2_subs = manager.get_client_subscriptions("client2").await;
    println!("\nCurrent subscriptions:");
    println!("Client 1: {:?}", client1_subs);
    println!("Client 2: {:?}", client2_subs);

    // Client 1 adds another symbol
    println!("\nClient 1 subscribing to NVDA");
    manager
        .subscribe_symbol("client1".to_string(), "NVDA".to_string())
        .await
        .unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Client 2 unsubscribes from one symbol
    println!("Client 2 unsubscribing from TSLA");
    manager
        .unsubscribe_symbol("client2".to_string(), "TSLA".to_string())
        .await
        .unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check updated subscriptions
    let client1_subs = manager.get_client_subscriptions("client1").await;
    let client2_subs = manager.get_client_subscriptions("client2").await;
    println!("\nUpdated subscriptions:");
    println!("Client 1: {:?}", client1_subs);
    println!("Client 2: {:?}", client2_subs);

    // Show active symbols across all clients
    let active_symbols = manager.get_active_symbols().await;
    println!("\nActive symbols across all clients: {:?}", active_symbols);

    // Wait for some market data to be received
    println!("\nWaiting for market data... (10 seconds)");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // Client 1 unsubscribes from all symbols
    println!("\nClient 1 unsubscribing from all symbols");
    manager
        .unsubscribe_all("client1".to_string())
        .await
        .unwrap();

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Final state
    let client1_subs = manager.get_client_subscriptions("client1").await;
    let client2_subs = manager.get_client_subscriptions("client2").await;
    let active_symbols = manager.get_active_symbols().await;

    println!("\nFinal state:");
    println!("Client 1: {:?}", client1_subs);
    println!("Client 2: {:?}", client2_subs);
    println!("Active symbols: {:?}", active_symbols);

    // Shutdown the manager
    println!("\nShutting down subscription manager...");
    cancellation_token.cancel();
    manager.close().await;

    // Wait for receiver tasks to complete
    let _ = tokio::join!(client1_handle, client2_handle);

    println!("Example completed successfully!");
}
