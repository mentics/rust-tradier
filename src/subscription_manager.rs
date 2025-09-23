use crate::http::tradier_post;
use chrono::NaiveDateTime;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::select;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::sync::CancellationToken;

/// Data structure sent to clients containing market data updates
#[derive(Debug, Clone)]
pub struct MarketData<T> {
    pub timestamp: NaiveDateTime,
    pub symbol: String,
    pub data: T
}

/// Client subscription state with data channel
#[derive(Debug)]
struct ClientSubscription<T> {
    symbols: HashSet<String>,
    data_tx: mpsc::Sender<MarketData<T>>
}

/// Main subscription manager that handles websocket connections and client subscriptions
///
/// This struct manages websocket connections to Tradier's live data API and allows multiple
/// clients to subscribe/unsubscribe to market data for different symbols. Data is delivered
/// through tokio channels instead of trait objects.
///
/// # Example
///
/// ```rust,no_run
/// use rust_tradier::LiveDataSubscriptionManager;
/// use tokio::sync::mpsc;
///
/// // Create the subscription manager
/// let manager = LiveDataSubscriptionManager::<String>::new();
///
/// // Create a channel for receiving data
/// let (data_tx, mut data_rx) = mpsc::channel(100);
///
/// // Subscribe to some symbols
/// manager.subscribe("client1".to_string(), vec!["AAPL".to_string(), "GOOGL".to_string()], data_tx.clone()).await;
///
/// // Subscribe to a single symbol (need to get existing channel)
/// manager.subscribe_symbol("client1".to_string(), "MSFT".to_string()).await;
///
/// // Receive data in a separate task
/// tokio::spawn(async move {
///     while let Some(market_data) = data_rx.recv().await {
///         println!("Received data for {}: {:?}", market_data.symbol, market_data.data);
///     }
/// });
///
/// // Check current subscriptions
/// let subscriptions = manager.get_client_subscriptions("client1");
/// println!("Client1 is subscribed to: {:?}", subscriptions);
///
/// // Unsubscribe from a symbol
/// manager.unsubscribe_symbol("client1".to_string(), "GOOGL".to_string()).await;
///
/// // Unsubscribe from all symbols for this client
/// manager.unsubscribe_all("client1".to_string()).await;
///
/// // Shutdown the manager
/// manager.shutdown().await;
/// ```
#[derive(Debug)]
pub struct LiveDataSubscriptionManager<T> {
    clients: Arc<RwLock<HashMap<String, ClientSubscription<T>>>>,
    websocket_symbols: Arc<RwLock<HashSet<String>>>,
    websocket_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    cancellation_token: CancellationToken
}

impl<T> Default for LiveDataSubscriptionManager<T>
where
    T: Send + Sync + Clone + From<String> + 'static
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LiveDataSubscriptionManager<T>
where
    T: Send + Sync + Clone + From<String> + 'static
{
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            websocket_symbols: Arc::new(RwLock::new(HashSet::new())),
            websocket_task: Arc::new(RwLock::new(None)),
            cancellation_token: CancellationToken::new()
        }
    }

    /// Subscribe to symbols for a client
    pub async fn subscribe(
        &self,
        client_id: String,
        symbols: Vec<String>,
        data_tx: mpsc::Sender<MarketData<T>>
    ) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        let mut websocket_symbols = self.websocket_symbols.write().await;

        // Get or create client subscription
        let client = clients
            .entry(client_id.clone())
            .or_insert(ClientSubscription {
                symbols: HashSet::new(),
                data_tx
            });

        // Add symbols to client
        for symbol in &symbols {
            client.symbols.insert(symbol.clone());
            websocket_symbols.insert(symbol.clone());
        }

        println!("Client {} subscribed to symbols: {:?}", client_id, symbols);

        // Start websocket task if not running and we have symbols
        self.ensure_websocket_running().await;

        Ok(())
    }

    /// Subscribe to a single symbol for a client
    pub async fn subscribe_symbol(&self, client_id: String, symbol: String) -> Result<(), String> {
        // Get the existing client's data channel
        let data_tx = {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&client_id) {
                client.data_tx.clone()
            } else {
                return Err(format!(
                    "Client {} not found. Must subscribe to symbols first.",
                    client_id
                ));
            }
        };

        self.subscribe(client_id, vec![symbol], data_tx).await
    }

    /// Unsubscribe from symbols for a client
    pub async fn unsubscribe(&self, client_id: String, symbols: Vec<String>) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        let mut websocket_symbols = self.websocket_symbols.write().await;

        if let Some(client) = clients.get_mut(&client_id) {
            // Remove symbols from client
            for symbol in &symbols {
                client.symbols.remove(symbol);
            }

            // Clean up websocket symbols - only remove if no clients are subscribed
            let mut symbols_to_remove = Vec::new();
            for symbol in &symbols {
                let mut still_needed = false;
                // Collect client data first to avoid borrowing issues
                let client_data: Vec<(String, HashSet<String>)> = clients
                    .iter()
                    .filter(|(cid, _)| *cid != &client_id)
                    .map(|(cid, client)| (cid.clone(), client.symbols.clone()))
                    .collect();

                for (_cid, client_symbols) in client_data {
                    if client_symbols.contains(symbol) {
                        still_needed = true;
                        break;
                    }
                }
                if !still_needed {
                    symbols_to_remove.push(symbol.clone());
                }
            }

            for symbol in symbols_to_remove {
                websocket_symbols.remove(&symbol);
            }
        }

        println!(
            "Client {} unsubscribed from symbols: {:?}",
            client_id, symbols
        );
        Ok(())
    }

    /// Unsubscribe from a single symbol for a client
    pub async fn unsubscribe_symbol(
        &self,
        client_id: String,
        symbol: String
    ) -> Result<(), String> {
        self.unsubscribe(client_id, vec![symbol]).await
    }

    /// Unsubscribe from all symbols for a client
    pub async fn unsubscribe_all(&self, client_id: String) -> Result<(), String> {
        let mut clients = self.clients.write().await;
        let mut websocket_symbols = self.websocket_symbols.write().await;

        let symbols_to_remove = if let Some(client) = clients.get_mut(&client_id) {
            let symbols: Vec<String> = client.symbols.iter().cloned().collect();

            // Remove symbols from this client first
            for symbol in &symbols {
                client.symbols.remove(symbol);
            }
            symbols
        } else {
            Vec::new()
        };

        // Now check which symbols can be removed from websocket subscription
        // We need to release the mutable borrow before doing this
        drop(clients);
        let clients = self.clients.read().await;

        for symbol in &symbols_to_remove {
            // Check if other clients still need this symbol
            let mut still_needed = false;
            for (cid, other_client) in clients.iter() {
                if cid != &client_id && other_client.symbols.contains(symbol) {
                    still_needed = true;
                    break;
                }
            }

            if !still_needed {
                websocket_symbols.remove(symbol);
            }
        }

        println!("Client {} unsubscribed from all symbols", client_id);
        Ok(())
    }

    /// Get currently subscribed symbols for a client
    pub async fn get_client_subscriptions(&self, client_id: &str) -> Vec<String> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(client_id) {
            client.symbols.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all currently active symbols across all clients
    pub async fn get_active_symbols(&self) -> Vec<String> {
        let websocket_symbols = self.websocket_symbols.read().await;
        websocket_symbols.iter().cloned().collect()
    }

    /// Shutdown the subscription manager
    pub async fn close(&self) {
        // Signal cancellation to stop the websocket task
        self.cancellation_token.cancel();

        // Wait for the websocket task to finish if it's running
        let mut websocket_task = self.websocket_task.write().await;
        if let Some(task) = websocket_task.take() {
            let _ = task.await;
        }

        println!("Subscription manager shutdown complete");
    }

    /// Ensure websocket task is running if we have symbols to subscribe to
    async fn ensure_websocket_running(&self) {
        let should_start = {
            let websocket_task = self.websocket_task.read().await;
            websocket_task.is_none()
        };

        if should_start {
            self.start_websocket_task().await;
        }
    }

    /// Start the websocket connection task
    async fn start_websocket_task(&self) {
        println!("Starting websocket connection task");

        let clients = Arc::clone(&self.clients);
        let websocket_symbols = Arc::clone(&self.websocket_symbols);
        let cancellation_token = self.cancellation_token.clone();

        let task = tokio::spawn(async move {
            Self::run_websocket_task(clients, websocket_symbols, cancellation_token).await;
        });

        let mut websocket_task = self.websocket_task.write().await;
        *websocket_task = Some(task);
    }

    /// Run the websocket connection and data handling task
    async fn run_websocket_task(
        clients: Arc<RwLock<HashMap<String, ClientSubscription<T>>>>,
        websocket_symbols: Arc<RwLock<HashSet<String>>>,
        cancellation_token: CancellationToken
    ) where
        T: From<String>
    {
        println!("Starting websocket connection task");

        loop {
            // Wait for symbols to be available or cancellation
            let symbols = loop {
                let current_symbols: Vec<String> = {
                    let websocket_symbols_lock = websocket_symbols.read().await;
                    websocket_symbols_lock.iter().cloned().collect()
                };

                if !current_symbols.is_empty() {
                    break current_symbols;
                }

                // Wait for either cancellation or a short timeout to check again
                select! {
                    _ = cancellation_token.cancelled() => {
                        println!("Websocket task cancelled");
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Check again for symbols
                    }
                }
            };

            // Attempt to connect and run websocket session with cancellation
            let session_token = cancellation_token.clone();
            let should_continue =
                Self::run_websocket_session(&symbols, Arc::clone(&clients), session_token).await;

            if !should_continue {
                // Check if we should restart or exit
                let should_exit = {
                    let websocket_symbols_lock = websocket_symbols.read().await;
                    websocket_symbols_lock.is_empty()
                };

                if should_exit {
                    println!("No more symbols to subscribe to, exiting websocket task");
                    break;
                }

                // Wait before reconnecting, but allow cancellation
                select! {
                    _ = cancellation_token.cancelled() => {
                        println!("Websocket task cancelled during reconnect wait");
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        // Continue to reconnect
                    }
                }
            }

            // Check if we've been cancelled
            if cancellation_token.is_cancelled() {
                println!("Websocket task cancelled");
                return;
            }
        }
    }

    /// Run a single websocket session
    async fn run_websocket_session(
        symbols: &[String],
        clients: Arc<RwLock<HashMap<String, ClientSubscription<T>>>>,
        cancellation_token: CancellationToken
    ) -> bool
    where
        T: From<String>
    {
        println!("Connecting to websocket for symbols: {:?}", symbols);

        // Connect to websocket
        let (sid, ws_stream) = match Self::connect().await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to connect to websocket: {:?}", e);
                return true; // Try to reconnect
            }
        };

        let (mut write, mut read) = ws_stream.split();

        // Send subscription message
        let symbol_refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
        let payload =
            json!({ "symbols": symbol_refs, "sessionid": sid, "linebreak": false }).to_string();

        if let Err(e) = write.send(Message::Text(payload)).await {
            println!("Error sending subscription: {:?}", e);
            return true; // Try to reconnect
        }

        println!("Successfully subscribed to {} symbols", symbols.len());

        // Main message loop
        loop {
            println!("Waiting for message");
            let ping_timer = tokio::time::sleep(Duration::from_secs(100));

            select! {
                _ = cancellation_token.cancelled() => {
                    println!("Websocket session cancelled");
                    return false; // Don't try to reconnect
                }
                message = read.next() => {
                    match message {
                        Some(Ok(Message::Text(payload))) => {
                            println!("Received message: {}", payload);
                            Self::process_message(&payload, &clients).await;
                        }
                        Some(Ok(Message::Ping(payload))) => {
                            if let Err(e) = write.send(Message::Pong(payload)).await {
                                println!("Error sending pong: {:?}", e);
                                return true;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            println!("Received close message from websocket");
                            return true; // Try to reconnect
                        }
                        None => {
                            println!("Websocket stream ended");
                            return true;
                        }
                        Some(Err(e)) => {
                            println!("Websocket error: {:?}", e);
                            return true;
                        }
                        _ => {} // Ignore other message types
                    }
                }
                _ = ping_timer => {
                    // Timeout - send ping
                    if let Err(e) = write.send(Message::Ping(Vec::new())).await {
                        println!("Error sending ping: {:?}", e);
                        return true;
                    }
                }
            }
        }
    }

    /// Process incoming websocket message
    async fn process_message(
        payload: &str,
        clients: &Arc<RwLock<HashMap<String, ClientSubscription<T>>>>
    ) where
        T: From<String>
    {
        // Try to parse the message as JSON
        match serde_json::from_str::<Value>(payload) {
            Ok(json_value) => {
                // Extract symbol from the message if possible
                if let Some(symbol) = Self::extract_symbol_from_message(&json_value) {
                    // Collect client senders first to avoid holding lock across await
                    let client_senders: Vec<mpsc::Sender<MarketData<T>>> = {
                        let clients_lock = clients.read().await;
                        clients_lock
                            .iter()
                            .filter(|(_, client)| client.symbols.contains(&symbol))
                            .map(|(_, client)| client.data_tx.clone())
                            .collect()
                    };

                    // Create market data
                    let market_data = MarketData {
                        timestamp: chrono::Utc::now().naive_utc(),
                        symbol: symbol.clone(),
                        data: T::from(payload.to_string())
                    };

                    // Send data to all subscribed clients
                    for sender in client_senders {
                        let _ = sender.send(market_data.clone()).await;
                    }
                }
            }
            Err(e) => {
                println!("Failed to parse websocket message as JSON: {:?}", e);
            }
        }
    }

    /// Extract symbol from websocket message (placeholder implementation)
    fn extract_symbol_from_message(json_value: &Value) -> Option<String> {
        // This is a placeholder - you'd need to implement the actual parsing
        // based on Tradier's websocket message format
        json_value
            .get("symbol")
            .and_then(|s| s.as_str())
            .map(|symbol| symbol.to_string())
    }

    /// Connect to Tradier websocket
    async fn connect() -> Result<
        (
            String,
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
            >
        ),
        Box<dyn std::error::Error + Send + Sync>
    > {
        let resp = tradier_post("/markets/events/session", None).await?;
        let data: Value = serde_json::from_str(&resp)?;
        let s = &data["stream"];
        let sid = s["sessionid"].as_str().unwrap().to_string();
        let url = "wss://ws.tradier.com/v1/markets/events";

        let url_parsed = reqwest::Url::parse(url)?;
        let (ws_stream, _) = connect_async(url_parsed).await?;

        Ok((sid, ws_stream))
    }
}

/// Example handler implementation for demonstration
pub struct PrintHandler;

impl PrintHandler {
    pub async fn run_receiver(mut data_rx: mpsc::Receiver<MarketData<String>>) {
        while let Some(market_data) = data_rx.recv().await {
            println!(
                "{}: Received data for {}: {}",
                market_data.timestamp, market_data.symbol, market_data.data
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscription_manager() {
        // Set a dummy API key to avoid the error
        std::env::set_var("TRADIER_API_KEY", "test_key");

        let manager = LiveDataSubscriptionManager::<String>::new();

        // Create a channel for receiving data
        let (data_tx, _data_rx) = mpsc::channel(100);

        // Subscribe to some symbols
        manager
            .subscribe(
                "client1".to_string(),
                vec!["AAPL".to_string(), "GOOGL".to_string()],
                data_tx.clone()
            )
            .await
            .unwrap();

        // Check subscriptions
        let subscriptions = manager.get_client_subscriptions("client1").await;
        assert_eq!(subscriptions.len(), 2);
        assert!(subscriptions.contains(&"AAPL".to_string()));
        assert!(subscriptions.contains(&"GOOGL".to_string()));

        // Subscribe to more symbols
        manager
            .subscribe_symbol("client1".to_string(), "MSFT".to_string())
            .await
            .unwrap();

        let subscriptions = manager.get_client_subscriptions("client1").await;
        assert_eq!(subscriptions.len(), 3);
        assert!(subscriptions.contains(&"MSFT".to_string()));

        // Unsubscribe from one symbol
        manager
            .unsubscribe_symbol("client1".to_string(), "GOOGL".to_string())
            .await
            .unwrap();

        let subscriptions = manager.get_client_subscriptions("client1").await;
        assert_eq!(subscriptions.len(), 2);
        assert!(!subscriptions.contains(&"GOOGL".to_string()));

        // Unsubscribe from all
        manager
            .unsubscribe_all("client1".to_string())
            .await
            .unwrap();

        let subscriptions = manager.get_client_subscriptions("client1").await;
        assert_eq!(subscriptions.len(), 0);

        // Shutdown
        manager.close().await;
    }
}
