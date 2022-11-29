use super::{Hash, Key, Storage};

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::Write, net::TcpStream};

const STUDENT_NAME: &str = "Elina Safarova";

pub fn handle_connection(mut stream: TcpStream, storage: Storage) {
    let greeting = serde_json::to_vec(&json!({ "student_name": STUDENT_NAME })).unwrap();
    stream.write_all(&greeting).unwrap();
    let mut de = serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
    loop {
        let query = Query::deserialize(&mut de);
        let response = match query {
            Ok(query) => form_response(query, storage.clone()),
            Err(err) => {
                if err.is_eof() || err.is_io() {
                    error!("couldn't read the request {err}");
                    panic!("{err}");
                } else {
                    json!({ "response_status": Response::INVALID_REQUEST })
                }
            }
        };
        info!("prepared response {response}");
        let response = serde_json::to_vec(&response).unwrap();

        stream.write_all(&response).unwrap();
    }
}

fn form_response(query: Query, storage: Storage) -> Value {
    let Query {
        request_type,
        key,
        hash,
    } = query;
    match request_type.as_str() {
        Query::STORE_QUERY => {
            let hash = hash.unwrap();
            info!("got store request with key=\"{key}\" and value=\"{hash}\"");
            let mut storage_guard = storage.lock().unwrap();
            storage_guard.insert(key, hash);
            json!({ "response_status": Response::SUCCESS })
        }
        Query::LOAD_QUERY => {
            let storage_guard = storage.lock().unwrap();
            if let Some(hash) = { storage_guard.get(&key) } {
                info!("got load request with key=\"{key}\"");
                json!({
                    "response_status": Response::SUCCESS,
                    "requested_key" : key,
                    "requested_hash": hash,
                })
            } else {
                json!({ "response_status": Response::NO_KEY })
            }
        }
        _ => {
            json!({ "response_status": Response::INVALID_REQUEST })
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Query {
    pub request_type: String,
    pub key: Key,
    #[serde(default)]
    pub hash: Option<Hash>,
}

impl Query {
    const LOAD_QUERY: &str = "load";
    const STORE_QUERY: &str = "store";
}
struct Response;

impl Response {
    const SUCCESS: &str = "success";
    const NO_KEY: &str = "key not found";
    const INVALID_REQUEST: &str = "invalid request";
}
