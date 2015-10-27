extern crate iron;

use std::error::Error;
use std::io;
use std::sync::mpsc;

use postgres;
use r2d2;
use r2d2_postgres;
use r2d2_postgres::PostgresConnectionManager as PCM;


pub type Pool = r2d2::Pool<PCM>;
pub type Config = r2d2::Config<postgres::Connection, r2d2_postgres::Error>;
type PooledConnnection = r2d2::PooledConnection<PCM>;

pub fn pool(url: &str, config: r2d2::Config<postgres::Connection, r2d2_postgres::Error>) -> Pool {
    let mgr = PCM::new(url, postgres::SslMode::None).unwrap();
    r2d2::Pool::new(config, mgr).unwrap()
}

pub struct PGResponseReader {
    // lazy_rows: Option<postgres::rows::LazyRows<'a, 'b>>,
    pub rx: mpsc::Receiver<String>,
    pub started: bool,
    pub last_event: Option<Result<String, mpsc::RecvError>>,
}

impl PGResponseReader {
    pub fn new(rx: mpsc::Receiver<String>) -> PGResponseReader {

        PGResponseReader {
            rx: rx,
            started: false,
            last_event: None,
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
            self.last_event = None;
        } else {
            event_json_msg = self.rx.recv();
        }

        // assume that each event len is < the buf len (65K)
        while buf_write_idx < buf.len() && !(event_json_msg.clone()).is_err() {
            let event_json = event_json_msg.clone().unwrap();
            let event_json_bytes = event_json.as_bytes();
            if event_json_msg.is_err() {
                break;
            }
            if event_json.len() + buf_write_idx > buf.len() {
                self.last_event = Some(event_json_msg.clone());
                break;
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
