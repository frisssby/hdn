use serde::{de::DeserializeOwned, Serialize};
use serde_json::de::{Deserializer, IoRead};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    ops::DerefMut,
    sync::Mutex,
};

type JsonReader = Deserializer<IoRead<TcpStream>>;
type JsonReaderError = serde_json::Error;

pub trait HdnAgent {
    fn new(addr: SocketAddr, stream: TcpStream) -> Self;
    fn send<M: Serialize>(&mut self, message: &M);
    fn try_read<M: DeserializeOwned>(&mut self) -> Result<M, JsonReaderError>;
}

pub struct Client {
    pub addr: SocketAddr,
    stream: TcpStream,
    reader: JsonReader,
}

impl HdnAgent for Client {
    fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        let reader = JsonReader::from_reader(stream.try_clone().unwrap());
        Self {
            addr,
            stream,
            reader,
        }
    }

    fn send<M: Serialize>(&mut self, message: &M) {
        let data = serde_json::to_vec(&message).unwrap();
        self.stream.write_all(&data).unwrap();
    }

    fn try_read<M: DeserializeOwned>(&mut self) -> Result<M, JsonReaderError> {
        M::deserialize(&mut self.reader)
    }
}

pub struct Peer {
    pub addr: SocketAddr,
    stream: Mutex<TcpStream>,
    reader: Mutex<JsonReader>,
}

impl Clone for Peer {
    fn clone(&self) -> Self {
        let stream = self.stream.lock().unwrap().try_clone().unwrap();
        Peer::new(self.addr, stream)
    }
}

impl HdnAgent for Peer {
    fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        let reader = JsonReader::from_reader(stream.try_clone().unwrap());
        Self {
            addr,
            stream: Mutex::new(stream),
            reader: Mutex::new(reader),
        }
    }

    fn send<M: Serialize>(&mut self, message: &M) {
        let data = serde_json::to_vec(&message).unwrap();
        let mut stream = self.stream.lock().unwrap();
        stream.write_all(&data).unwrap();
    }

    fn try_read<M: DeserializeOwned>(&mut self) -> Result<M, JsonReaderError> {
        let mut reader = self.reader.lock().unwrap();
        M::deserialize(reader.deref_mut())
    }
}
