use serde::{Deserialize, Serialize};
use chrono::DateTime;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]

pub struct SingleSwap {
    pub ID : i64,
    pub CreatedAt: chrono::DateTime<chrono::Utc>,
    pub UpdatedAt: chrono::DateTime<chrono::Utc>,
    pub initiatorAddress : String,
    pub redeemerAddress : Option<String>,
    pub chain : String,
    pub asset : String,
    pub amount : String,
    pub priceByOracle : f64
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MatchedOrder {
    pub ID : i64,
    pub CreatedAt: DateTime<chrono::Utc>,
    pub UpdatedAt: DateTime<chrono::Utc>,
    pub InitiatorAtomicSwapID : i64,
    pub FollowerAtomicSwapID : i64,
    pub initiatorAtomicSwap : SingleSwap,
    pub followerAtomicSwap: SingleSwap,
    pub userBtcWalletAddress : Option<String>,
    pub tags : Option<Vec<String>>,
    pub maker : String,
    pub taker : String,
    pub orderPair : String,
    pub status : i32
}

#[derive(Serialize,Debug,Deserialize,FromRow)]
pub struct UserTags {
    pub address : String,
    pub tags : Vec<String>
}