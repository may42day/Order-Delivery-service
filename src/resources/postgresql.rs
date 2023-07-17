use async_graphql::Context;
use bb8::RunError;
use diesel::result::Error;
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager, PoolError},
    AsyncPgConnection,
};

use crate::utils::configs::Config;

pub async fn establish_connection_pool(database_url: String) -> DbPool {
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_url);
    Pool::builder()
        .build(config)
        .await
        .expect("Cannot build database pool")
}

pub type DbPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConn<'a> = bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

// pub async fn execute_connection<'a>(context: &'a Context<'a>) -> DbConn<'a> {
pub async fn execute_connection<'a>(
    context: &'a Context<'a>,
) -> Result<DbConn<'a>, RunError<PoolError>> {
    let db_pool = &context
        .data::<Config>()
        .expect("Cannot get DbPool from context")
        .db_pool;
    db_pool.get().await
}
