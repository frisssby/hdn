use crate::{
    agent::{Client, HdnAgent, Peer},
    messages::{Greeting, Load, PeerUpdate, Request, Response, Store},
    storage::{Storage, Time},
};

use log::error;
use serde::Deserialize;
use std::{
    collections::HashSet,
    env, fs,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};
use threadpool::ThreadPool;

#[derive(Deserialize, Debug)]
pub struct NodeConfig {
    pub user: String,
    pub nodes: Vec<IpAddr>,
    pub id: usize,
    pub client_port: u16,
    pub peer_port: u16,
}

impl NodeConfig {
    pub fn build() -> NodeConfig {
        let config_path = env::var("HDN_CONFIG").unwrap();
        let data = fs::read(config_path).expect("failed to read configuration file");
        serde_json::from_slice(&data).expect("failed to parse configuration file")
    }
}

#[derive(Clone)]
pub struct Node {
    pub username: String,
    storage: Arc<Storage>,
    client_socket: SocketAddr,
    peers: Arc<Mutex<Vec<Peer>>>,
}

impl Node {
    const MAX_CLIENTS: usize = 100;

    pub fn init(config: NodeConfig) -> Self {
        let peers = setup_network(&config);
        assert!(peers.lock().unwrap().len() + 1 == config.nodes.len());
        let ip = config.nodes[config.id];

        Node {
            username: config.user,
            storage: Arc::new(Storage::new()),
            client_socket: SocketAddr::new(ip, config.client_port),
            peers,
        }
    }

    pub fn launch(&self) {
        let peers = self.peers.lock().unwrap();
        let pool = ThreadPool::new(Node::MAX_CLIENTS + peers.len());
        for peer in peers.iter().cloned() {
            let mut node = self.clone();
            pool.execute(move || {
                node.handle_peer(peer);
            });
        }
        drop(peers);

        let listener = TcpListener::bind(self.client_socket).unwrap();
        loop {
            let connection = listener.accept();
            match connection {
                Ok((stream, addr)) => {
                    let mut node = self.clone();
                    pool.execute(move || {
                        node.handle_client(Client::new(addr, stream));
                    });
                }
                Err(err) => {
                    error!("failed to accept connection from client: {err}");
                }
            }
        }
    }

    fn handle_client(&mut self, mut client: Client) {
        client.send(&Greeting {
            student_name: &self.username,
        });
        loop {
            let request = client.try_read::<Request>();
            let response = match request {
                Ok(request) => match request {
                    Request::Store(request) => self.on_store_request(&request),
                    Request::Load(request) => self.on_load_request(&request),
                },
                Err(err) => {
                    if err.is_eof() || err.is_io() {
                        panic!("{err}");
                    } else {
                        Response::InvalidRequest
                    }
                }
            };
            client.send(&response);
        }
    }

    fn handle_peer(&mut self, mut peer: Peer) {
        loop {
            let update = peer.try_read::<PeerUpdate>().unwrap();
            self.storage
                .synchronize(update.key, update.hash, update.time);
        }
    }

    fn on_store_request(&mut self, request: &Store) -> Response {
        let time = self
            .storage
            .store(request.key.clone(), request.hash.clone());
        self.notify_peers(request, time);
        Response::Success
    }

    fn on_load_request(&self, request: &Load) -> Response {
        if let Some(hash) = { self.storage.load(&request.key) } {
            Response::Data {
                requested_key: request.key.clone(),
                requested_hash: hash,
            }
        } else {
            Response::KeyNotFound
        }
    }

    fn notify_peers(&self, request: &Store, time: Time) {
        let mut peers = self.peers.lock().unwrap();
        for peer in peers.iter_mut() {
            peer.send(&PeerUpdate {
                key: request.key.clone(),
                hash: request.hash.clone(),
                time,
            });
        }
    }
}

fn setup_network(config: &NodeConfig) -> Arc<Mutex<Vec<Peer>>> {
    let (nodes, id, port) = (&config.nodes, config.id, config.peer_port);
    if nodes.len() < 2 {
        return Arc::new(Mutex::new(Vec::new()));
    }

    let peers = Arc::new(Mutex::new(Vec::new()));
    let pool = ThreadPool::new(nodes.len() - 1);
    for ip in nodes[id + 1..].iter() {
        let addr = SocketAddr::new(*ip, port);
        let peers = peers.clone();
        pool.execute(move || connect_to_peer(addr, peers));
    }

    let peer_listener = TcpListener::bind(SocketAddr::new(nodes[id], port)).unwrap();
    let mut awaited = HashSet::from_iter(nodes[0..id].iter().cloned());
    while !awaited.is_empty() {
        if let Some(peer) = try_accept_peer_connection(&peer_listener, &awaited, peers.clone()) {
            awaited.remove(&peer);
        }
    }
    pool.join();
    peers
}

fn connect_to_peer(addr: SocketAddr, peers: Arc<Mutex<Vec<Peer>>>) {
    loop {
        const TIMEOUT_SECS: u64 = 3;
        let connection = TcpStream::connect_timeout(&addr, Duration::from_secs(TIMEOUT_SECS));
        if let Ok(stream) = connection {
            let mut peers = peers.lock().unwrap();
            peers.push(Peer::new(addr, stream));
            break;
        }
    }
}

fn try_accept_peer_connection(
    listener: &TcpListener,
    nodes: &HashSet<IpAddr>,
    peers: Arc<Mutex<Vec<Peer>>>,
) -> Option<IpAddr> {
    let connection = listener.accept();
    if let Ok((stream, addr)) = connection {
        let ip = addr.ip();
        if nodes.contains(&ip) {
            let mut peers = peers.lock().unwrap();
            peers.push(Peer::new(addr, stream));
            return Some(ip);
        }
    }
    None
}
