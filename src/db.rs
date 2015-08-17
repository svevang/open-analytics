extern crate iron;
use std::error::Error;

use r2d2;
use r2d2_postgres;
use r2d2_postgres::PostgresConnectionManager as PCM;
use postgres;

pub type Pool = r2d2::Pool<PCM>;
pub type Config = r2d2::Config<postgres::Connection, r2d2_postgres::Error>;
type PooledConnnection = r2d2::PooledConnection<PCM>;

pub fn pool(url: &str, config: r2d2::Config<postgres::Connection, r2d2_postgres::Error>) -> Pool {
    let mgr = PCM::new(url, postgres::SslMode::None).unwrap();
    r2d2::Pool::new(config, mgr).unwrap()
}

