use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::MatchedOrder;

#[derive(Error,Debug,Serialize,Deserialize)]
pub enum OrderBookError{
    #[error("Error Deserializing orders")]
    DeserializationError,

    #[error("Error Fetching orders from Orderbook")]
    FetchError,
}

pub struct OrderBook{
    client : Client
}

impl OrderBook{
    pub async fn  new() -> Result<Self,OrderBookError> {
        let client : Client = Client::new();
        Ok(OrderBook{
            client
        })
    }

    pub async fn fetch_orders(&self) -> Result<Vec<MatchedOrder>, OrderBookError> {
        let response = self.client.get("https://api.garden.finance/orders?verbose=true")
            .send()
            .await
            .map_err(|_| OrderBookError::FetchError)?;

        if response.status().is_success() {
            let orders: Vec<MatchedOrder> = match response.json().await {
                Ok(orders)=>orders,
                Err(err) => {
                    println!("Error While Deserializing : {:?}",err);
                    return Err(OrderBookError::DeserializationError)
                }
            };
            Ok(orders)
        } else {
            Err(OrderBookError::FetchError)
        }
    }
    
}
