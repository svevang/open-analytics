mod db;
mod app;

extern crate postgres;
extern crate iron;
extern crate router;
extern crate time;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rustc_serialize;
extern crate bodyparser;
extern crate persistent;

use persistent::Read;
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use router::Router;
use postgres::Connection;
use rustc_serialize::json;

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
    json: json::Json

}


fn print_database(conn:r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>){

    println!("hi there!");
    let stmt = conn.prepare("SELECT * FROM analytics").unwrap();
    for row in stmt.query(&[]).unwrap() {
        let id:i32 = row.get::<_, i32>(0);
        let name:String =  row.get::<_, String>(1);
        let json=  row.get::<_, rustc_serialize::json::Json>(2);
        println!("Found event {}, {}, {:?}", id, name, json);
        let event = Event {
            id: id,
            name: name,
            json: json
        };
    }

}

fn event_read(req: &mut Request) -> IronResult<Response> {
    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();
    print_database(conn);
    let ref namespace = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");
    Ok(Response::with((iron::status::Ok, *namespace)))
}

fn event_write(req: &mut Request) -> IronResult<Response> {
    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();

    // https://github.com/iron/body-parser
    let body = req.get::<bodyparser::Json>();
    match body {
        Ok(Some(body)) => {
            let ref namespace = (req.extensions.get::<Router>()
                                 .unwrap().find("name").unwrap_or("missing name param"));

            let resp = format!("Post: {}", *namespace);
            println!("Read body:\n{}", body);
            conn.execute("INSERT INTO analytics (name, data) VALUES ($1, $2)",
                             &[namespace, &body]).unwrap();
            print_database(conn);
            Ok(Response::with((iron::status::Ok, resp)))
        },
        Ok(None) => {
            Ok(Response::with((iron::status::BadRequest, "empty-body")))
        },
        Err(err) => {
            Ok(Response::with((iron::status::BadRequest, "no body")))
        }
    }

}

const MAX_BODY_LENGTH: usize = 1024 * 5;

fn main() {

    let app = app::App::new();

    let mut router = Router::new();
    let arc_app = Arc::new(app);

    let mut chain = Chain::new(event_read);
    chain.link_before(app::AppMiddleware::new(arc_app.clone()));
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    router.get("api/v1/:name", chain);

    let mut chain = Chain::new(event_write);
    chain.link_before(ResponseTime);
    chain.link_before(app::AppMiddleware::new(arc_app.clone()));
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain.link_after(ResponseTime);
    router.post("api/v1/:name", chain);

    println!("starting server on localhost:3000");
    Iron::new(router).http("localhost:3000").unwrap();

}
