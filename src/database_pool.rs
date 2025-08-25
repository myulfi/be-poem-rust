use sqlx::Pool;
use sqlx::{MySql, Postgres};

use sqlx::{mysql::MySqlRow, postgres::PgRow};

pub enum DatabasePool {
    Postgres(Pool<Postgres>),
    MySql(Pool<MySql>),
}

impl DatabasePool {
    pub async fn fetch_all_postgres(&self, query: &str) -> sqlx::Result<Vec<PgRow>> {
        match self {
            DatabasePool::Postgres(pool) => sqlx::query(query).fetch_all(pool).await,
            _ => Err(sqlx::Error::Protocol("Not a Postgres pool".into())),
        }
    }

    pub async fn fetch_all_mysql(&self, query: &str) -> sqlx::Result<Vec<MySqlRow>> {
        match self {
            DatabasePool::MySql(pool) => sqlx::query(query).fetch_all(pool).await,
            _ => Err(sqlx::Error::Protocol("Not a MySQL pool".into())),
        }
    }

    pub async fn execute(&self, query: &str) -> sqlx::Result<u64> {
        match self {
            DatabasePool::Postgres(pool) => {
                let res = sqlx::query(query).execute(pool).await?;
                Ok(res.rows_affected())
            }
            DatabasePool::MySql(pool) => {
                let res = sqlx::query(query).execute(pool).await?;
                Ok(res.rows_affected())
            }
        }
    }
}
