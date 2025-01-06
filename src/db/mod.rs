use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};

use crate::models::{BtcClosingPrice, ChainflipSwap, ThorchainSwap, UserTags};

#[derive(Clone)]
pub struct TagDB {
    pool: Pool<Postgres>,
}

impl TagDB {
    pub async fn init() -> Result<Self, sqlx::error::Error> {
        dotenv().ok();

        let postgres_url = std::env::var("POSTGRES_URL").expect("POSTGRES_URL is required in ENV");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&postgres_url)
            .await?;

        Ok(TagDB { pool })
    }

    pub async fn add_tag(
        &self,
        address: &String,
        tag: &String,
    ) -> Result<UserTags, sqlx::error::Error> {
        let query = r#"
            INSERT INTO user_tags (address, tags)
            VALUES ($1, ARRAY[$2]::text[])
            ON CONFLICT (address)
            DO UPDATE SET tags = array_append(user_tags.tags, $2)
            WHERE NOT $2 = ANY(user_tags.tags)
            RETURNING address, tags;
        "#;

        let row = sqlx::query(query)
            .bind(&address)
            .bind(&tag)
            .fetch_one(&self.pool)
            .await?;

        Ok(UserTags {
            address: row.get("address"),
            tags: row.get("tags"),
        })
    }

    pub async fn get_user_tags(&self, address: &String) -> Result<UserTags, sqlx::error::Error> {
        let query = r#"
            SELECT address, tags FROM user_tags WHERE address = $1;
        "#;

        if let Ok(row) = sqlx::query(query)
            .bind(&address)
            .fetch_one(&self.pool)
            .await
        {
            Ok(UserTags {
                address: row.get("address"),
                tags: row.get("tags"),
            })
        } else {
            Ok(UserTags {
                address: address.to_owned(),
                tags: Vec::new(),
            })
        }
    }

    pub async fn get_chainflip_swaps(&self) -> Result<Vec<ChainflipSwap>, sqlx::Error> {
        let query = r#"
            SELECT
                timestamp,
                CASE
                    WHEN in_asset = 'BTC' THEN in_amount
                    ELSE out_amount
                END AS btc_amount,
                CASE
                    WHEN in_asset = 'BTC' THEN in_address
                    ELSE out_address
                END AS btc_address,
                CASE
                    WHEN in_asset = 'BTC' THEN in_amount_usd::DOUBLE PRECISION
                    ELSE out_amount_usd::DOUBLE PRECISION
                END AS usd_amount
                FROM chainflip_swaps
                WHERE (in_asset = 'BTC' OR out_asset = 'BTC') and timestamp>=1727740800
        "#;

        let swaps: Vec<ChainflipSwap> = sqlx::query_as::<_, ChainflipSwap>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(swaps)
    }

    pub async fn get_thorchain_swaps(&self) -> Result<Vec<ThorchainSwap>, sqlx::Error> {
        let query = r#"
        SELECT
            timestamp,
            CASE
                WHEN in_asset = 'BTC.BTC' THEN in_amount::DOUBLE PRECISION
                ELSE out_amount_1::DOUBLE PRECISION
            END AS btc_amount,
            CASE
                WHEN in_asset = 'BTC.BTC' THEN in_address
                ELSE out_address_1
            END AS btc_address
            FROM native_swaps_thorchain 
            WHERE (in_asset = 'BTC.BTC' OR out_asset_1 = 'BTC.BTC') and timestamp>=1727740800
        "#;

        let swaps: Vec<ThorchainSwap> = sqlx::query_as::<_, ThorchainSwap>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(swaps)
    }

    pub async fn get_btc_closing_prices(&self) -> Result<Vec<BtcClosingPrice>, sqlx::Error> {
        let query = r#"
            SELECT
                date,
                closing_price_usd::DOUBLE PRECISION
            FROM btc_closing_prices
        "#;

        let closing_prices: Vec<BtcClosingPrice> = sqlx::query_as::<_, BtcClosingPrice>(query)
            .fetch_all(&self.pool)
            .await?;

        Ok(closing_prices)
    }
}
