use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::TradierError;
use serde::de::{self, Deserializer};

fn deserialize_orders<'de, D>(deserializer: D) -> Result<Option<Orders>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    if s == "null" {
        Ok(None)
    } else {
        let orders: Orders = serde_json::from_str(&s).map_err(de::Error::custom)?;
        Ok(Some(orders))
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct OrdersResponse {
    #[serde(deserialize_with = "deserialize_orders")]
    pub orders: Option<Orders>,
}

// impl OrdersResponse {
//     fn deserialize_orders<'de, D>(deserializer: D) -> Result<Option<Orders>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s: String = String::deserialize(deserializer)?;
//         if s == "null" {
//             Ok(None)
//         } else {
//             let orders: Orders = serde_json::from_str(&s).map_err(de::Error::custom)?;
//             Ok(Some(orders))
//         }
//     }
// }

#[derive(Debug,Serialize,Deserialize)]
pub struct Orders {
    pub order: Vec<Order>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Order {
    pub id: i64,
    pub order_type: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub status: String,
    pub duration: String,
    pub price: Option<f64>,
    pub avg_fill_price: f64,
    pub exec_quantity: f64,
    pub last_fill_price: f64,
    pub last_fill_quantity: f64,
    pub remaining_quantity: f64,
    pub create_date: DateTime<Utc>,
    pub transaction_date: DateTime<Utc>,
    pub class: String,
    pub option_symbol: Option<String>,
    pub strategy: Option<String>,
    pub num_legs: Option<i32>,
    pub leg: Option<Vec<OrderLeg>>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct OrderLeg {
    pub id: i64,
    #[serde(rename = "type")]
    pub order_type: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub status: String,
    pub duration: String,
    pub price: Option<f64>,
    pub avg_fill_price: f64,
    pub exec_quantity: f64,
    pub last_fill_price: f64,
    pub last_fill_quantity: f64,
    pub remaining_quantity: f64,
    pub create_date: DateTime<Utc>,
    pub transaction_date: DateTime<Utc>,
    pub class: String,
    pub option_symbol: Option<String>,
}

pub async fn fetch_orders(account_id: &str) -> Result<Option<Vec<Order>>, TradierError> {
    let response = fetch_orders_json(account_id).await?;
    let res: OrdersResponse = serde_json::from_value(response)?;
    Ok(res.orders.map(|orders| orders.order))
}

pub async fn fetch_orders_json(account_id: &str) -> Result<serde_json::Value, TradierError> {
    // TODO: pagination?
    let url = format!("/accounts/{}/orders?includeTags=true", account_id);
    println!("{}", url);
    let response = crate::http::tradier_get(&url).await?;
    let json_value: serde_json::Value = serde_json::from_str(&response)?;
    Ok(json_value)
}
