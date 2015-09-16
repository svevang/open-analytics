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

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use iron::response::ResponseBody;
use iron::response::WriteBody;
use iron::modifier::Modifier;
use time::precise_time_ns;
use chrono::*;
use router::Router;
use rustc_serialize::json;
use rustc_serialize::json::ToJson;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::RecvError;
use std::io;
use std::thread;

struct BoxRead(Box<io::Read + Send>);
impl WriteBody for BoxRead {
    fn write_body(&mut self, b: &mut ResponseBody) -> io::Result<()> {
        io::copy(&mut self.0, b)
            .map(|_| ())
    }
}

impl Modifier<Response> for BoxRead {
    fn modify(self, res: &mut Response) {
        res.body = Some(Box::new(self));
    }
}


struct PGResponseReader{
    //lazy_rows: Option<postgres::rows::LazyRows<'a, 'b>>,
    rx: std::sync::mpsc::Receiver<String>,
    started: bool,
    last_event: Option<Result<String, RecvError>>
}

impl PGResponseReader {
    fn new(rx:std::sync::mpsc::Receiver<String>) -> PGResponseReader{

        PGResponseReader{
            rx: rx,
            started: false,
            last_event: None

        }
    }

}

impl io::Read for PGResponseReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

        self.started = true;
        let mut buf_write_idx = 0;

        let mut event_json_msg;

        // prime the first json blob for the main loop
        if self.last_event.is_some() {
            let event_json_msg_optional = self.last_event.clone();
            event_json_msg = event_json_msg_optional.unwrap();
        }
        else{
            event_json_msg = self.rx.recv();
        }

        // assume that each event len is < the buf len (65K)
        while buf_write_idx < buf.len() && !(event_json_msg.clone()).is_err(){
            let event_json = event_json_msg.clone().unwrap();
            let event_json_bytes = event_json.as_bytes();
            if event_json_msg.is_err(){
                break
            }
            if event_json.len() + buf_write_idx > buf.len(){
                self.last_event = Some(event_json_msg.clone());
                break
            }
            for i in 0..event_json.len() {
                buf[i + buf_write_idx] = event_json_bytes[i];
            }
            buf_write_idx += event_json.len();


            event_json_msg = self.rx.recv();
        }
        Ok(buf_write_idx)
    }
}

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

fn event_read(req: &mut Request) -> IronResult<Response> {

    let before = precise_time_ns();
    let conn = req.extensions.get::<app::App>().unwrap().database.get().unwrap();
    let delta = precise_time_ns() - before;
    println!("Fetching database handle took: {} ms", (delta as f64) / 1000000.0);


    let ref id_param = req.extensions.get::<Router>()
        .unwrap().find("id").unwrap_or("missing name param");
    let id = id_param.parse::<i32>().unwrap();
    let ref namespace = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");


    let stmt = conn.prepare("SELECT id, event_data, name, date_created FROM analytics where name=$1 and id=$2 limit 1").unwrap();
    let result = stmt.query(&[namespace, &id]).unwrap();

    if result.len() == 1 {
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
    else{
        Ok(Response::with((iron::status::NotFound, "[]")))
    }
}
fn event_list(req: &mut Request) -> IronResult<Response> {

    let ref namespace = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");
    let name = namespace.parse::<String>().unwrap();
    let (tx, rx) = sync_channel::<String>(0);
    let db_pool = req.extensions.get::<app::App>().unwrap().database.clone();
    let conn = db_pool.get().unwrap();
    thread::spawn(move|| {

        let trans = conn.transaction().unwrap();
        let stmt = conn.prepare("SELECT id, event_data, name, date_created FROM analytics where name=$1").unwrap();
        let result = stmt.lazy_query( &trans, &[&&name[..]], 500).unwrap();

        let mut started = false;

        if tx.send(String::from("[")).is_err() { return };

        for row in result {
            if started {
                if tx.send(String::from(",")).is_err() { return };;
            }
            started = true;
            let row = row.unwrap();
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
            let event_json = event.to_json().to_string();
            if tx.send(event_json).is_err() { return };
        }
        if tx.send(String::from("]")).is_err() { return };
    });

    let reader = Box::new(PGResponseReader::new(rx));
    Ok(Response::with((iron::status::Ok, BoxRead(reader) )))
    //Ok(Response::with((iron::status::Ok, events.to_json().to_string())))
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
    chain.link_before(persistent::Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
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
