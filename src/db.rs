use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    println!("Set Manager DB");

    let pool = r2d2::Pool::builder()
        .build(manager)
        .map_err(|e| anyhow::anyhow!("Failed to create pool: {}", e))?;
    println!("Set Pool DB");
    Ok(pool)
}
