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
extern crate chrono;

use persistent::Read;
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use chrono::*;
use router::Router;
use rustc_serialize::json;
use rustc_serialize::json::ToJson;

use std::collections::HashMap;
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
#[derive(RustcEncodable)]
struct Event {
    id: i32,
    name: String,
    event_data: json::Json,
    date_created: DateTime<UTC>
}

// JSON value representation
impl json::ToJson for Event {
    fn to_json(&self) -> json::Json {

        let mut d = HashMap::new();
        d.insert("id".to_string(), self.id.to_json());
        d.insert("name".to_string(), self.name.to_json());
        d.insert("date_created".to_string(), self.date_created.format("%+").to_string().to_json());

        d.to_json()
    }
}

fn print_database(conn:r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>){

    println!("hi there!");
    let stmt = conn.prepare("SELECT id, name, event_data, date_created FROM analytics").unwrap();
    for row in stmt.query(&[]).unwrap() {
        let id:i32 = row.get::<_, i32>(0);
        let name:String =  row.get::<_, String>(1);
        let event_data =  row.get::<_, rustc_serialize::json::Json>(2);
        let date_created =  row.get::<_, DateTime<UTC>>(3);
        let event = Event {
            id: id,
            name: name,
            event_data: event_data,
            date_created: date_created
        };
        println!("Found event {}, {}, {:?} {}", event.id, event.name, event.event_data, event.date_created);
    }

}

fn event_read(req: &mut Request) -> IronResult<Response> {

    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();

    let ref id_param = req.extensions.get::<Router>()
        .unwrap().find("id").unwrap_or("missing name param");
    let id = id_param.parse::<i32>().unwrap();
    let ref namespace = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");


    let stmt = conn.prepare("SELECT id, event_data, name, date_created FROM analytics where name=$1 and id=$2 limit 1").unwrap();
    let result = stmt.query(&[namespace, &id]).unwrap();

    let row = result.get(0);
    let id:i32 = row.get::<_, i32>(0);
    let event_data =  row.get::<_, rustc_serialize::json::Json>(1);
    let name:String =  row.get::<_, String>(2);
    let date_created =  row.get::<_, DateTime<UTC>>(3);
    let event = Event {
        id: id,
        name: name,
        event_data: event_data,
        date_created: date_created
    };
    Ok(Response::with((iron::status::Ok, event.to_json().to_string())))
}
fn event_list(req: &mut Request) -> IronResult<Response> {
    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();
    let stmt = conn.prepare("SELECT id, event_data, name, date_created FROM analytics where name=$1").unwrap();
    let ref namespace = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");
    let result = stmt.query(&[namespace]).unwrap();
    let mut events:Vec<rustc_serialize::json::Json> = Vec::new();
    for row in result {
        let id:i32 = row.get::<_, i32>(0);
        let event_data =  row.get::<_, rustc_serialize::json::Json>(1);
        let name:String =  row.get::<_, String>(2);
        let date_created =  row.get::<_, DateTime<UTC>>(3);
        let event = Event {
            id: id,
            name: name,
            event_data: event_data,
            date_created: date_created
        };
        events.push(event.to_json());
    }
    //let json_response = events.pop().unwrap();
    Ok(Response::with((iron::status::Ok, events.to_json().to_string())))
}

fn event_write(req: &mut Request) -> IronResult<Response> {
    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();

    // https://github.com/iron/body-parser
    let body = req.get::<bodyparser::Json>();
    match body {
        Ok(Some(body)) => {
            let ref namespace = req.extensions.get::<Router>()
                                 .unwrap().find("name").unwrap_or("missing name param");

            let stmt = conn.prepare("INSERT INTO analytics (name, event_data) VALUES ($1, $2) RETURNING id").unwrap();
            let result = stmt.query(&[namespace, &body]).unwrap();
            let row_id:i32 = result.get(0).get::<_, i32>(0);
            print_database(req.extensions.get::<app::App>().unwrap().database.get().unwrap());
            Ok(Response::with((iron::status::Ok, format!("{{\"id\": \"{}\"}}", row_id))))
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

    let mut chain = Chain::new(event_list);
    chain.link_before(app::AppMiddleware::new(arc_app.clone()));
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    router.get("api/v1/events/:name", chain);

    let mut chain = Chain::new(event_write);
    chain.link_before(ResponseTime);
    chain.link_before(app::AppMiddleware::new(arc_app.clone()));
    chain.link_before(Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain.link_after(ResponseTime);
    router.post("api/v1/events/:name/new", chain);

    let mut chain = Chain::new(event_read);
    chain.link_before(ResponseTime);
    chain.link_before(app::AppMiddleware::new(arc_app.clone()));
    chain.link_after(ResponseTime);
    router.get("api/v1/events/:name/:id", chain);

    println!("starting server on localhost:3000");
    Iron::new(router).http("localhost:3000").unwrap();

}
