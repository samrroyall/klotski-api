use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool as R2D2Pool};

fn get_database_url() -> String {
    dotenvy::dotenv().ok();

    let db_host = dotenvy::var("POSTGRES_HOST").expect("POSTGRES_HOST is not set");
    let db_port = dotenvy::var("POSTGRES_PORT").expect("POSTGRES_PORT is not set");
    let db_username = dotenvy::var("POSTGRES_USERNAME").expect("POSTGRES_USERNAME is not set");
    let db_password = dotenvy::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD is not set");
    let db_name = dotenvy::var("POSTGRES_DATABASE").expect("POSTGRES_DATABASE is not set");

    format!("postgres://{db_username}:{db_password}@{db_host}:{db_port}/{db_name}")
}

pub type Pool = R2D2Pool<ConnectionManager<PgConnection>>;

pub fn get_db_pool() -> Pool {
    let database_url = get_database_url();

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::new(manager).expect("Failed to create DB pool.")
}
