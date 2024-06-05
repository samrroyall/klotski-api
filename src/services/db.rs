use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool as R2D2Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn get_db_url() -> String {
    dotenvy::dotenv().ok();

    let db_host = dotenvy::var("PG_HOST").expect("PG_HOST is not set");
    let db_port = dotenvy::var("PG_PORT").expect("PG_PORT is not set");
    let db_username = dotenvy::var("PG_USERNAME").expect("PG_USERNAME is not set");
    let db_password = dotenvy::var("PG_PASSWORD").expect("PG_PASSWORD is not set");
    let db_name = dotenvy::var("PG_DATABASE").expect("PG_DATABASE is not set");

    format!("postgres://{db_username}:{db_password}@{db_host}:{db_port}/{db_name}")
}

pub type Pool = R2D2Pool<ConnectionManager<PgConnection>>;

pub fn get_db_pool() -> Pool {
    let database_url = get_db_url();

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::new(manager).expect("Failed to create DB pool.")
}

pub fn run_migrations(conn: &mut impl MigrationHarness<Pg>) {
    tracing::info!("Running db migrations");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Diesel migrations failed");
}
