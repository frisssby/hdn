use serde::de::Deserialize;
use serde_json::{de::IoRead, json, Value};
use serial_test::serial;
use std::{
    io::{self, prelude::*},
    net::{IpAddr, Shutdown, SocketAddr, TcpStream},
    process::{Child, Command},
    str::{self, FromStr},
    thread,
    time::Duration,
};

const BINARY_PATH: &str = env!("CARGO_BIN_EXE_hdn");
const STUDENT_NAME: &str = "Elina Safarova";

fn take_a_nap() {
    thread::sleep(Duration::from_millis(100));
}

struct ServerWrapper {
    process: Option<Child>,
    address: SocketAddr,
}

impl ServerWrapper {
    fn start() -> Self {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        let port = 43000u16;
        let process = Command::new(BINARY_PATH).spawn().unwrap();
        take_a_nap();
        Self {
            process: Some(process),
            address: SocketAddr::new(ip, port),
        }
    }

    fn is_alive(&mut self) -> bool {
        self.process
            .as_mut()
            .map_or(false, |proc| proc.try_wait().unwrap().is_none())
    }

    fn stop(&mut self) -> io::Result<()> {
        self.process.take().map_or(Ok(()), |mut proc| proc.kill())
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        self.stop().unwrap()
    }
}

struct Client {
    connection: TcpStream,
    deserializer: serde_json::Deserializer<IoRead<TcpStream>>,
}

impl Client {
    fn start(server_addr: SocketAddr) -> io::Result<Self> {
        let connection = TcpStream::connect(server_addr)?;
        let de = serde_json::Deserializer::from_reader(connection.try_clone()?);
        Ok(Self {
            connection,
            deserializer: de,
        })
    }

    fn make_request(&mut self, request: &Value) -> io::Result<()> {
        let data = serde_json::to_vec(request).unwrap();
        self.send(&data)
    }

    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.connection.write_all(&data)
    }

    fn load(&mut self, key: &str) -> io::Result<()> {
        let request = json!({
            "request_type": "load",
            "key": key,
        });
        self.make_request(&request)
    }

    fn store(&mut self, key: &str, value: &str) -> io::Result<()> {
        let request = json!({
            "request_type": "store",
            "key": key,
            "hash": value,
        });
        self.make_request(&request)
    }

    fn expect_response(&mut self, expected: &Value) {
        let response = self.get_response().unwrap();
        assert_eq!(&response, expected);
    }

    fn expect_no_response(&mut self) {
        assert_eq!(self.get_response(), None);
    }

    fn get_response(&mut self) -> Option<Value> {
        Value::deserialize(&mut self.deserializer).ok()
    }

    fn shutdown(&mut self, how: Shutdown) {
        let _ = self.connection.shutdown(how);
    }
}

fn successful_load(key: &str, hash: &str) -> Value {
    json!({
        "response_status": "success",
        "requested_key": key,
        "requested_hash": hash,
    })
}

fn key_not_found() -> Value {
    json!({
        "response_status": "key not found",
    })
}

fn successful_store() -> Value {
    json!({
        "response_status": "success",
    })
}

fn successful_connection() -> Value {
    json!({
        "student_name": STUDENT_NAME,
    })
}

fn invalid_request() -> Value {
    json!({
        "response_status": "invalid request",
    })
}

#[test]
#[serial(timeout_ms = 1000)]
fn test_one_client() {
    let server = ServerWrapper::start();

    let mut client = Client::start(server.address).unwrap();
    client.expect_response(&successful_connection());

    client.store("foo", "oof").unwrap();
    client.expect_response(&successful_store());

    client.load("foo").unwrap();
    client.expect_response(&successful_load("foo", "oof"));

    client.load("bar").unwrap();
    client.expect_response(&key_not_found());
}

#[test]
#[serial(timeout_ms = 1000)]
fn test_multiple_clients() {
    let server = ServerWrapper::start();

    let mut london = Client::start(server.address).unwrap();
    london.expect_response(&successful_connection());

    let mut paris = Client::start(server.address).unwrap();
    paris.expect_response(&successful_connection());

    let mut moscow = Client::start(server.address).unwrap();
    moscow.expect_response(&successful_connection());

    london.store("foo", "oof").unwrap();
    london.expect_response(&successful_store());

    paris.load("foo").unwrap();
    paris.expect_response(&successful_load("foo", "oof"));

    moscow.store("bar", "rab").unwrap();
    moscow.expect_response(&successful_store());

    london.load("bar").unwrap();
    london.expect_response(&successful_load("bar", "rab"));
}

#[test]
#[serial(timeout_ms = 1000)]
fn test_invalid_request() {
    let mut server = ServerWrapper::start();

    let mut client = Client::start(server.address).unwrap();
    client.expect_response(&successful_connection());

    client
        .make_request(&json!({
            "request_type": "say",
            "key": "meow",
        }))
        .unwrap();

    client.expect_response(&invalid_request());
    assert!(server.is_alive());

    client.send(b"{\"request_type\":\"load\"}").unwrap();
    client.expect_response(&invalid_request());
    assert!(server.is_alive());

    client.store("answer", "42").unwrap();
    client.expect_response(&successful_store());
}

#[test]
#[serial(timeout_ms = 1000)]

fn test_client_close_write_connection() {
    let mut server = ServerWrapper::start();

    let mut client = Client::start(server.address).unwrap();
    client.expect_response(&successful_connection());

    client.load("some_key").unwrap();
    client.shutdown(Shutdown::Write);
    take_a_nap();
    assert!(server.is_alive());
}

#[test]
#[serial(timeout_ms = 1000)]

fn test_client_close_both_connections() {
    let mut server = ServerWrapper::start();

    let mut client = Client::start(server.address).unwrap();
    client.shutdown(Shutdown::Both);
    take_a_nap();
    client.expect_no_response();
    assert!(server.is_alive());
}

#[test]
#[serial(timeout_ms = 1000)]
fn test_tricky_symbols() {
    let server = ServerWrapper::start();

    let mut client = Client::start(server.address).unwrap();
    client.expect_response(&successful_connection());

    client.store("{key}", "{hash}").unwrap();
    client.expect_response(&successful_store());

    client.load("{key}").unwrap();
    client.expect_response(&successful_load("{key}", "{hash}"));

    client.store("\"key\"", "{\"hash\"}").unwrap();
    client.expect_response(&successful_store());

    client.load("key").unwrap();
    client.expect_response(&key_not_found());
}
