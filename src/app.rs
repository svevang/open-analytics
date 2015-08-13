use r2d2;
use db;
use iron;
use iron::Request;
use iron::{BeforeMiddleware};
use std::sync::Arc;

pub struct App {
    pub database: db::Pool,
}

impl App {
    pub fn new(config: &db::Config) -> App {

        let db_config = r2d2::Config::builder()
            .pool_size(2)
            .helper_threads(2)
            .build();

        return App {
            database: db::pool("postgres://sam@localhost/open_analytics_development", db_config),
        };
    }
}

impl iron::typemap::Key for App {
    type Value = App;
}

pub struct AppMiddleware {
        app: Arc<App>
}

impl BeforeMiddleware for AppMiddleware {
    fn before(&self, req: &mut Request) -> Result<(), iron::error::IronError> {
        if !req.extensions.contains::<App>() {
            let app = req.extensions.get::<App>();
            //req.extensions.insert::<db::DatabasePool>(db::DatabasePool::new(app));
        }
        Ok(())
    }
}
