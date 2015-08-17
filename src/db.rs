extern crate iron;
use std::cell::Cell;
use std::error::Error;
use std::mem;
use std::sync::Arc;

use r2d2;
use r2d2_postgres;
use r2d2_postgres::PostgresConnectionManager as PCM;
use iron::{BeforeMiddleware};
use iron::prelude::*;
use postgres;

use app::App;

pub type Pool = r2d2::Pool<PCM>;
pub type Config = r2d2::Config<postgres::Connection, r2d2_postgres::Error>;
type PooledConnnection = r2d2::PooledConnection<PCM>;

pub fn pool(url: &str, config: r2d2::Config<postgres::Connection, r2d2_postgres::Error>) -> Pool {
    let mgr = PCM::new(url, postgres::SslMode::None).unwrap();
    r2d2::Pool::new(config, mgr).unwrap()
}

pub struct DatabasePoolMiddleware;

pub struct DatabasePool {
    slot: r2d2::PooledConnection<PCM>,
    // Keep a handle to the app which keeps a handle to the database to ensure
    // that this `'static` is indeed at least a little more accurate (in that
    // it's alive for the period of time this `DatabasePool` is alive.
    app: Arc<App>,
}

impl DatabasePool {
    pub fn new(&self, app: Arc<App>) -> DatabasePool {

        let conn: PooledConnnection = self.app.database.get().unwrap();
        DatabasePool {
            app: app,
            slot: conn
        }
    }

}

impl BeforeMiddleware for DatabasePoolMiddleware {
    fn before(&self, req: &mut Request) -> Result<(), iron::error::IronError> {
        if !req.extensions.contains::<DatabasePool>() {
            let app = req.extensions.get::<App>();
           // println!("Hello! from database middlware {}", app.database)
           // req.extensions.insert::<App>(self.app.clone());
           // let database_pool = DatabasePool::new(app.unwrap());
           // req.extensions.insert::<DatabasePool>(database_pool);
        }
        Ok(())
    }
}

impl iron::typemap::Key for DatabasePool {
    type Value = DatabasePool;
}
