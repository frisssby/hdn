mod client_handler;

use log::error;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, TcpListener},
    sync::{Arc, Mutex},
};
use threadpool::ThreadPool;

pub type Key = String;
pub type Hash = String;
type Storage = Arc<Mutex<HashMap<Key, Hash>>>;

fn create_storage() -> Storage {
    Arc::new(Mutex::new(HashMap::<Key, Hash>::new()))
}

pub fn run(ip: IpAddr, port: u16) {
    const MAX_CLIENTS: usize = 100;
    let pool = ThreadPool::new(MAX_CLIENTS);
    let listener = TcpListener::bind(SocketAddr::new(ip, port)).unwrap();
    let storage = create_storage();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let storage = storage.clone();
                pool.execute(move || client_handler::handle_connection(stream, storage));
            }
            Err(err) => {
                error!("failed to accept connection: {err}");
            }
        }
    }
}
