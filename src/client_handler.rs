use super::{Hash, Key, Storage};

use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{io::Write, net::TcpStream};

#[derive(Serialize, Deserialize)]
#[serde(tag = "request_type", rename_all = "lowercase")]
enum Request {
    Load { key: Key },
    Store { key: Key, hash: Hash },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "response_status")]
enum Response {
    #[serde(rename = "success")]
    Data {
        requested_key: Key,
        requested_hash: Hash,
    },
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "key not found")]
    KeyNotFound,
    #[serde(rename = "invalid request")]
    InvalidRequest,
}

pub fn handle_connection(mut stream: TcpStream, storage: Storage) {
    const STUDENT_NAME: &str = "Elina Safarova";
    let greeting = serde_json::to_vec(&json!({ "student_name": STUDENT_NAME })).unwrap();
    stream.write_all(&greeting).unwrap();

    let mut de = serde_json::Deserializer::from_reader(stream.try_clone().unwrap());
    loop {
        let query = Request::deserialize(&mut de);
        let response = match query {
            Ok(query) => form_response(query, storage.clone()),
            Err(err) => {
                if err.is_eof() || err.is_io() {
                    error!("couldn't read the request: {err}");
                    panic!("{err}");
                } else {
                    Response::InvalidRequest
                }
            }
        };
        let response = serde_json::to_value(response).unwrap();
        info!("prepared response {response}");
        let response = serde_json::to_vec(&response).unwrap();
        stream.write_all(&response).unwrap();
    }
}

fn form_response(request: Request, storage: Storage) -> Response {
    match request {
        Request::Store { key, hash } => {
            info!("got store request with key=\"{key}\" and value=\"{hash}\"");
            let mut storage_guard = storage.lock().unwrap();
            storage_guard.insert(key, hash);
            Response::Success
        }
        Request::Load { key } => {
            let storage_guard = storage.lock().unwrap();
            if let Some(hash) = { storage_guard.get(&key) } {
                info!("got load request with key=\"{key}\"");
                Response::Data {
                    requested_key: key,
                    requested_hash: hash.to_owned(),
                }
            } else {
                Response::KeyNotFound
            }
        }
    }
}
