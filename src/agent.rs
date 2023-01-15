use serde::{de::DeserializeOwned, Serialize};
use serde_json::de::{Deserializer, IoRead};
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

type JsonReader = Deserializer<IoRead<TcpStream>>;
type JsonReaderError = serde_json::Error;

pub struct HdnAgent {
    pub addr: SocketAddr,
    stream: TcpStream,
    reader: JsonReader,
}

impl HdnAgent {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        let reader = JsonReader::from_reader(stream.try_clone().unwrap());
        Self {
            addr,
            stream,
            reader,
        }
    }

    pub fn send<M: Serialize>(&mut self, message: &M) {
        let data = serde_json::to_vec(&message).unwrap();
        self.stream.write_all(&data).unwrap();
    }

    pub fn try_read<M: DeserializeOwned>(&mut self) -> Result<M, JsonReaderError> {
        M::deserialize(&mut self.reader)
    }
}

impl Clone for HdnAgent {
    fn clone(&self) -> Self {
        let stream = self.stream.try_clone().unwrap();
        HdnAgent::new(self.addr, stream)
    }
}
