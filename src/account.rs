use crate::error::TradierError;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};

fn deserialize_orders<'de, D>(deserializer: D) -> Result<Option<Orders>, D::Error>
where
    D: Deserializer<'de>
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::String(s) if s == "null" => Ok(None),
        Value::Object(orders) => orders
            .get("orders")
            .map(|o| serde_json::from_value(o.to_owned()))
            .transpose()
            .map_err(serde::de::Error::custom),
        _ => serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OneOrMore<T> {
    Multiple(Vec<T>),
    One(T)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdersResponse {
    #[serde(deserialize_with = "deserialize_orders")]
    pub orders: Option<Orders>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Orders {
    pub order: OneOrMore<Order>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Order {
    pub id: u64,
    pub tag: Option<String>,
    #[serde(rename = "type")]
    pub order_type: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f32,
    pub status: String,
    pub duration: String,
    pub price: Option<f32>,
    pub avg_fill_price: f32,
    pub exec_quantity: f32,
    pub last_fill_price: f32,
    pub last_fill_quantity: f32,
    pub remaining_quantity: f32,
    pub create_date: String,
    pub transaction_date: String,
    pub class: String,
    pub option_symbol: Option<String>,
    pub stop_price: Option<f32>,
    pub reason_description: Option<String>,
    pub strategy: Option<String>,
    pub num_legs: Option<u32>,
    pub leg: Option<Vec<OrderLeg>>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderLeg {
    pub id: u64,
    #[serde(rename = "type")]
    pub order_type: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f32,
    pub status: String,
    pub duration: String,
    pub price: Option<f32>,
    pub avg_fill_price: f32,
    pub exec_quantity: f32,
    pub last_fill_price: f32,
    pub last_fill_quantity: f32,
    pub remaining_quantity: f32,
    pub create_date: String,
    pub transaction_date: String,
    pub class: String,
    pub option_symbol: Option<String>
}

// pub fn to_orders(response: OrdersResponse) -> Option<Vec<Order>> {
//     match response {
//         OrdersResponse {
//             orders: Some(Orders {
//                 order: OneOrMore::Multiple(orders),
//             }),
//         } => Some(orders),
//         OrdersResponse {
//             orders: Some(Orders {
//                 order: OneOrMore::One(order),
//             }),
//         } => Some(vec![order]),
//         OrdersResponse { orders: None } => None,
//     }
// }

pub fn value_to_orders(mut json: serde_json::Value) -> Option<Vec<Order>> {
    let order = json.get_mut("orders")?.get_mut("order")?.take();
    match order {
        Value::Array(orders) => Some(
            orders
                .into_iter()
                .map(|o| serde_json::from_value(o).unwrap())
                .collect()
        ),
        Value::Object(order) => Some(vec![match from_value(Value::Object(order)) {
            Ok(order) => order,
            Err(e) => panic!("Error deserializing order: {:?}", e)
        }]),
        _ => panic!("Unexpected order value: {:?}", order)
    }
}

pub async fn fetch_orders(account_id: &str) -> Result<Option<Vec<Order>>, TradierError> {
    // TODO: pagination?
    let url = format!("/accounts/{}/orders?includeTags=true", account_id);
    let response = crate::http::tradier_get(&url).await?;
    let json_value: serde_json::Value = serde_json::from_str(&response)?;
    Ok(value_to_orders(json_value))
    // // let res: Result<OrdersResponse, serde_json::Error> = serde_json::from_value(response);
    // match json_value {
    //     Ok(orders_response) => Ok(value_to_orders(orders_response)),
    //     Err(e) => {
    //         eprintln!("Error deserializing orders response: {:?}", e);
    //         panic!("{:?}", e)
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ser() {
        let obj = test_obj();
        let json = serde_json::to_string(&obj).unwrap();
        println!("{}", json);
    }

    // #[test]
    // fn test_deser() {
    //     let response = include_str!("../data/response.json");
    //     println!("{}", response);
    //     let orders_response: OrdersResponse = serde_json::from_str(response).unwrap();
    // }

    // let obj = test_obj();
    // let json = serde_json::to_string(&obj).unwrap();
    // let orders_response: OrdersResponse = serde_json::from_str(&json).unwrap();
    // println!("{:?}", orders_response);

    // load file 'data/response.json' and parse it as OrdersResponse

    fn test_obj() -> OrdersResponse {
        let order = Order {
            id: 1,
            tag: Some("tag".to_string()),
            order_type: "order_type".to_string(),
            symbol: "symbol".to_string(),
            side: "side".to_string(),
            quantity: 1.0,
            status: "status".to_string(),
            duration: "duration".to_string(),
            price: Some(1.0),
            avg_fill_price: 1.0,
            exec_quantity: 1.0,
            last_fill_price: 1.0,
            last_fill_quantity: 1.0,
            remaining_quantity: 1.0,
            create_date: "2023-01-01T00:00:00Z".to_string(),
            transaction_date: "2023-01-01T00:00:00Z".to_string(),
            class: "class".to_string(),
            option_symbol: Some("option_symbol".to_string()),
            stop_price: Some(1.0),
            reason_description: Some("reason_description".to_string()),
            strategy: Some("strategy".to_string()),
            num_legs: Some(1),
            leg: Some(vec![OrderLeg {
                id: 1,
                order_type: "order_type".to_string(),
                symbol: "symbol".to_string(),
                side: "side".to_string(),
                quantity: 1.0,
                status: "status".to_string(),
                duration: "duration".to_string(),
                price: Some(1.0),
                avg_fill_price: 1.0,
                exec_quantity: 1.0,
                last_fill_price: 1.0,
                last_fill_quantity: 1.0,
                remaining_quantity: 1.0,
                create_date: "2023-01-01T00:00:00Z".to_string(),
                transaction_date: "2023-01-01T00:00:00Z".to_string(),
                class: "class".to_string(),
                option_symbol: Some("option_symbol".to_string())
            }])
        };
        let orders = Orders {
            order: OneOrMore::One(order)
        };
        OrdersResponse {
            orders: Some(orders)
        }
    }
}
