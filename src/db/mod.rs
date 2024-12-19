use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};

use crate::models::UserTags;

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

    pub async fn add_tag(&self, address: &String, tag: &String) -> Result<UserTags, sqlx::error::Error> {
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
                address : address.to_owned(),
                tags: Vec::new(),
            })
        }
    }
}
