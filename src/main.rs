extern crate postgres;
extern crate iron;
extern crate router;
extern crate time;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use router::{Router};
use postgres::{Connection, SslMode};

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

fn setup_database(conn:&Connection){

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

/*fn event_read(_: &mut Request) -> IronResult<Response> {
    let ref name = _.extensions.get::<Router>().unwrap().find(":name").unwrap_or("/");
    //Ok(Response::with((iron::status::Ok, format!("Hello: your event {}", name))));
    //Ok(Response::with(iron::status::Ok, *name));
    Ok(Response::with((iron::status::Ok, *name)));

}*/

fn event_read(req: &mut Request, db_conn: &Connection) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>()
        .unwrap().find("name").unwrap_or("missing name param");
    println!("{}",query);
    Ok(Response::with((iron::status::Ok, *query)))
}

fn wrap_handlers(handler: fn(req: &mut Request, &postgres::Connection), db_conn: &Connection) -> fn(req: &mut Request) -> IronResult<Response> {


    |req: &mut Request| -> IronResult<Response> {  handler(req, db_conn) }
}
fn main() {

    let conn = Connection::connect("postgres://sam@localhost/open_analytics_development", &SslMode::None)
            .unwrap();

    let mut router = Router::new();

    //let mut chain = Chain::new(event_read);
    let mut chain = Chain::new(wrap_handlers(event_read, &conn));
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);

    router.get("api/v1/:name", chain);
    Iron::new(router).http("localhost:3000").unwrap();
}
