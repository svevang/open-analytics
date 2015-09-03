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
    pub fn new() -> App {

        let db_config = r2d2::Config::builder()
            .pool_size(10)
            .helper_threads(20)
            .build();

        return App {
            database: db::pool("postgres://sam@localhost/open_analytics_development", db_config),
        };
    }
}

impl iron::typemap::Key for App {
    type Value = Arc<App>;
}

pub struct AppMiddleware {
        app: Arc<App>
}

impl AppMiddleware {
    pub fn new(app: Arc<App>) -> AppMiddleware {
        AppMiddleware { app: app }
    }
}

impl BeforeMiddleware for AppMiddleware {
    fn before(&self, req: &mut Request) -> Result<(), iron::error::IronError> {
        if !req.extensions.contains::<App>() {
            req.extensions.insert::<App>(self.app.clone());
        }
        Ok(())
    }

/*
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
    */
}
