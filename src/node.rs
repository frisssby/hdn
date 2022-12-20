use crate::{
    agent::HdnAgent,
    messages::{Greeting, Load, PeerUpdate, Request, Response, Store},
    storage::{Storage, Time},
};

use dns_lookup::lookup_host;
use log::{error, info};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use threadpool::ThreadPool;

#[derive(Deserialize, Debug)]
pub struct NetworkConfig {
    pub user: String,
    pub nodes: Vec<String>,
    pub client_port: u16,
    pub peer_port: u16,
}

impl NetworkConfig {
    pub fn build(config: &Path) -> Self {
        let data = fs::read(config).expect("fail to read configuration file");
        serde_json::from_slice(&data).expect("fail to parse configuration file")
    }
}

#[derive(Clone)]
pub struct Node {
    pub username: String,
    storage: Arc<Storage>,
    client_socket: SocketAddr,
    peers: Arc<Mutex<Vec<HdnAgent>>>,
}

fn resolve_ip(host: &str) -> Option<IpAddr> {
    host.parse().ok().or(lookup_host(host)
        .ok()
        .and_then(|ips| ips.into_iter().next()))
}

impl Node {
    const MAX_CLIENTS: usize = 100;

    pub fn init(config: NetworkConfig, node_id: usize) -> Self {
        info!("initialize server node with config {config:?} and id = {node_id}");

        let nodes: Vec<_> = config
            .nodes
            .iter()
            .map(|host| resolve_ip(host).expect("fail to resolve ip address"))
            .collect();

        let peers = setup_network(&nodes, config.peer_port, node_id);
        assert!(peers.lock().unwrap().len() + 1 == config.nodes.len());

        Node {
            username: config.user,
            storage: Arc::new(Storage::new()),
            client_socket: SocketAddr::new(nodes[node_id], config.client_port),
            peers,
        }
    }

    pub fn launch(&self) {
        info!("start server on {}", self.client_socket.ip());
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
                    info!("establish connection with client {}", addr.ip());
                    pool.execute(move || {
                        node.handle_client(HdnAgent::new(addr, stream));
                    });
                }
                Err(err) => {
                    error!("fail to accept connection from client: {err}");
                }
            }
        }
    }

    fn handle_client(&mut self, mut client: HdnAgent) {
        client.send(&Greeting {
            student_name: &self.username,
        });
        info!("send greeting to {}", client.addr.ip());
        loop {
            let request = client.try_read::<Request>();
            let response = match request {
                Ok(request) => match request {
                    Request::Store(request) => {
                        info!(
                            "receive request from client {} to store hash \"{}\" by key \"{}\"",
                            client.addr, request.hash, request.key
                        );
                        self.on_store_request(&request)
                    }
                    Request::Load(request) => {
                        info!(
                            "receive request from client {} to load hash by key \"{}\"",
                            client.addr, request.key
                        );
                        self.on_load_request(&request)
                    }
                },
                Err(err) => {
                    if err.is_eof() || err.is_io() {
                        error!("socket error from client {}: {}\n", client.addr.ip(), err);
                        panic!("{err}");
                    } else {
                        error!("");
                        Response::InvalidRequest
                    }
                }
            };
            client.send(&response);
            info!("send response: {response:?}")
        }
    }

    fn handle_peer(&mut self, mut peer: HdnAgent) {
        loop {
            let update = peer.try_read::<PeerUpdate>().unwrap();
            info!("receive update {:?} from peer {}", update, peer.addr.ip());
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

fn setup_network(nodes: &[IpAddr], peer_port: u16, node_id: usize) -> Arc<Mutex<Vec<HdnAgent>>> {
    if nodes.len() < 2 {
        return Arc::new(Mutex::new(Vec::new()));
    }

    let peers = Arc::new(Mutex::new(Vec::new()));
    let pool = ThreadPool::new(nodes.len() - 1);
    for ip in nodes[node_id + 1..].iter() {
        let addr = SocketAddr::new(*ip, peer_port);
        let peers = peers.clone();
        pool.execute(move || connect_to_peer(addr, peers));
    }

    let peer_listener = TcpListener::bind(SocketAddr::new(nodes[node_id], peer_port)).unwrap();
    let mut awaited = HashSet::from_iter(nodes[0..node_id].iter().cloned());
    while !awaited.is_empty() {
        if let Some(peer) = try_accept_peer_connection(&peer_listener, &awaited, peers.clone()) {
            awaited.remove(&peer);
        }
    }
    pool.join();
    peers
}

fn connect_to_peer(addr: SocketAddr, peers: Arc<Mutex<Vec<HdnAgent>>>) {
    loop {
        const TIMEOUT_SECS: u64 = 3;
        let connection = TcpStream::connect_timeout(&addr, Duration::from_secs(TIMEOUT_SECS));
        if let Ok(stream) = connection {
            info!("establish connection with peer {}", addr.ip());
            let mut peers = peers.lock().unwrap();
            peers.push(HdnAgent::new(addr, stream));
            break;
        }
    }
}

fn try_accept_peer_connection(
    listener: &TcpListener,
    nodes: &HashSet<IpAddr>,
    peers: Arc<Mutex<Vec<HdnAgent>>>,
) -> Option<IpAddr> {
    let connection = listener.accept();
    if let Ok((stream, addr)) = connection {
        let ip = addr.ip();
        if nodes.contains(&ip) {
            info!("establish connection with peer {}", ip);
            let mut peers = peers.lock().unwrap();
            peers.push(HdnAgent::new(addr, stream));
            return Some(ip);
        }
    }
    None
}
