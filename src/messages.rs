use crate::storage::{Hash, Key, Time};

use chrono::serde::ts_milliseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "request_type", rename_all = "lowercase")]
pub enum Request {
    Load(Load),
    Store(Store),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Load {
    pub key: Key,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Store {
    pub key: Key,
    pub hash: Hash,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "response_status")]
pub enum Response {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Greeting<'a> {
    pub student_name: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerUpdate {
    pub key: Key,
    pub hash: Hash,
    #[serde(with = "ts_milliseconds")]
    pub time: Time,
}
