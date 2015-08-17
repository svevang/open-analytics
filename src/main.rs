mod db;
mod app;

extern crate postgres;
extern crate iron;
extern crate router;
extern crate time;
extern crate r2d2;
extern crate r2d2_postgres;


use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use router::Router;
use postgres::Connection;

use std::sync::Arc;
struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

struct Event {
    id: i32,
    name: String,
    json: String
}


fn print_database(conn:&Connection){

    println!("hi there!");
    let stmt = conn.prepare("SELECT * FROM analytics").unwrap();
    for row in stmt.query(&[]).unwrap() {
        let event = Event {
            id: row.get(0),
            name: row.get(1),
            json: row.get(2)
        };
        println!("Found event {}", event.name);
    }

}

fn event_read(req: &mut Request) -> IronResult<Response> {
    let conn = req.extensions.get::<app::App>().unwrap().database.get();
    print_database(&conn.unwrap());
    let ref query = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");
    println!("{}",query);
    Ok(Response::with((iron::status::Ok, *query)))
}

fn main() {

    let app = app::App::new();

    let mut router = Router::new();

    let mut chain = Chain::new(event_read);
    chain.link_before(app::AppMiddleware::new(Arc::new(app)));
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);

    router.get("api/v1/:name", chain);
    println!("starting server on localhost:3000");
    Iron::new(router).http("localhost:3000").unwrap();

}
