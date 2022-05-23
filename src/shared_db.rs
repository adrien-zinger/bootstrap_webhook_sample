use hyper::{client::HttpConnector, Body, Client, Method, Request};
use serde::{Deserialize, Serialize};
use std::{cmp::min, collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;

use crate::{deserialize, to_bytes, MAX_CHUNK_SIZE};

#[derive(Deserialize, Serialize)]
pub enum EntryModif {
    Delete(String),
    Update((String, String)),
}

impl std::fmt::Debug for EntryModif {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryModif::Delete(d) => write!(f, "Delete({})", d),
            EntryModif::Update((k, v)) => write!(f, "Update({}, {})", k, v),
        }
    }
}

/// Structure for the listener of the [DB]
#[derive(Default)]
pub struct Subscriber {
    /// entry point to the subscriber,
    /// ex: http://127.0.0.1:3000/insert
    pub addr: String,
    /// start position index of the bootstrap
    pub begin: usize,
    /// last position index of the bootstrap
    pub end: usize,
    /// true if `end` initialized on `db.size()`, false otherwise
    /// if true, use `db.data.len()` instead of the `end` param
    pub eof: bool,
    /// current index of bootstrap, increment periodically.
    pub index: usize,
}

impl Subscriber {
    pub fn new(addr: String, begin: usize, end: usize) -> Self {
        Self {
            addr,
            index: begin,
            begin,
            end,
            eof: false, // updated on `add_subscriber`
        }
    }
}

#[derive(Default)]
pub struct DB {
    /// database, BTreeMap for index ordering
    data: BTreeMap<String, String>,
    /// Vector of subscribers listening the local DB
    subscribers: Vec<Subscriber>,
}

#[derive(Clone, Default)]
pub struct SharedDB(Arc<Mutex<DB>>);

impl SharedDB {
    /// Remove a value from the database
    pub async fn remove(&self, key: &String) {
        let mut guard = self.0.lock().await;
        println!("remove key: {}", key);
        guard.data.remove(key);

        // inform the bootstrapers of an update
        let pos = guard.data.iter().position(|(k, _)| *k == *key).unwrap();
        for subscriber in guard.subscribers.iter() {
            let end = match subscriber.eof {
                true => guard.data.len(),
                false => subscriber.end,
            };
            if pos >= subscriber.begin && end >= pos && pos <= subscriber.index {
                println!("forward remove: {}--{}", &subscriber.addr, key);
                forward_all(&subscriber.addr, &vec![EntryModif::Delete(key.clone())]).await;
            }
        }
    }

    /// Update something in the database, insert if doesn't exist
    pub async fn update(&self, key: &String, value: &str) {
        let mut guard = self.0.lock().await;
        println!("insert key: {} {}", key, value);
        guard.data.insert(key.clone(), value.to_string());

        // inform the bootstrapers of an update
        let pos = guard.data.iter().position(|(k, _)| *k == *key).unwrap();
        for subscriber in guard.subscribers.iter() {
            let end = match subscriber.eof {
                true => guard.data.len(),
                false => subscriber.end,
            };
            if pos >= subscriber.begin && end >= pos {
                // && pos <= subscriber.index {
                println!("forward update {}: {} {}", &subscriber.addr, key, value);
                forward_all(
                    &subscriber.addr,
                    &vec![EntryModif::Update((key.clone(), value.to_string()))],
                )
                .await;
            }
        }
        println!("quit insert function")
    }

    /// Add a bootstrap client subscriber
    pub async fn add_subscriber(&self, mut subscriber: Subscriber) {
        println!("add subscriber: {}", &subscriber.addr);
        let mut guard = self.0.lock().await;
        if guard.data.len() <= subscriber.end {
            subscriber.eof = true;
            subscriber.end = 10000000;
        }
        guard.subscribers.push(subscriber);
        println!("subscriber added");
    }

    /// Send for each subscriber what they need, if a subscriber has finished
    /// a bootstrap, we keep it in memory (for now) and continue to send
    /// information of the range he asked for.
    pub async fn send_chunks(&self) {
        // todo, better subscription management can be defined here
        // - remove the oldests subscribers (maybe an auto remove if stale)
        // - limit the number of subscriptions
        // - put all forward_all in an UnorderedFutures
        //   list to profit of concurrency
        // - send a progression status to the remote
        let guard = &mut *self.0.lock().await;
        for subscriber in guard.subscribers.iter_mut() {
            if subscriber.index == subscriber.end {
                println!("pass subscriber {}", &subscriber.addr);
                continue;
            }
            let chunk_size = min(MAX_CHUNK_SIZE, subscriber.end - subscriber.index);
            let chunk_modifs = take_chunk(&guard.data, subscriber.index, chunk_size);
            println!("send chunk {}: {:?}", &subscriber.addr, &chunk_modifs);
            forward_all(&subscriber.addr, &chunk_modifs).await;
            println!("forward chunk successed");
            subscriber.index += chunk_size;
        }
        println!("chunks sent");
    }

    pub async fn len(&self) -> usize {
        self.0.lock().await.data.len()
    }

    pub async fn dump(&self) {
        print!("dump: ");
        for (key, value) in self.0.lock().await.data.iter() {
            print!("{key}: {value};");
        }
        println!();
    }
}

fn take_chunk(data: &BTreeMap<String, String>, from: usize, size: usize) -> Vec<EntryModif> {
    data.iter()
        .skip(from)
        .take(size)
        .map(|(k, v)| EntryModif::Update((k.clone(), v.clone())))
        .collect()
}

/// Forward all modifications in `modifs` to the distant bootstraper client
/// at `addr` (uri address)
pub async fn forward_all(addr: &str, modifs: &Vec<EntryModif>) {
    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(addr)
        .body(Body::from(serde_json::to_string(modifs).unwrap()))
        .unwrap();
    client.request(req).await.unwrap();
}

#[allow(dead_code)]
/// Subscribe only to one target address
pub async fn subscribe(to_addr: String, my_addr: String) {
    let client = Client::new();
    let size = size_request(&client, &to_addr).await;
    // 0: begin, size: end
    //
    subscribe_request(&client, &to_addr, &my_addr, 0, size).await;
}

#[allow(dead_code)]
pub async fn subscribe_multiple(to_addr: &[String], my_addr: String) {
    assert!(!to_addr.is_empty());
    let client = Client::new();
    let size = size_request(&client, &to_addr[0]).await;
    let c = size / to_addr.len();
    let mut i = 0;
    let mut it_target_addr = to_addr.iter();
    while i < size {
        let t = it_target_addr.next().unwrap();
        if i + (c << 1) >= size {
            // todo: fix that, we should send an eof signal instead of a big
            // random value
            println!("big require to {}, for {} to {}", t, i, size << 1);
            subscribe_request(&client, t, &my_addr, i, size << 1).await;
            break;
        } else {
            println!("require to {}, for {} to {}", t, i, i + c);
            subscribe_request(&client, t, &my_addr, i, i + c).await;
        }
        i += c;
    }
}

async fn size_request(client: &Client<HttpConnector>, target: &String) -> usize {
    let res = client
        .get(format!("http://{}/size", target).parse().unwrap())
        .await
        .unwrap();
    deserialize(&to_bytes(res.into_body()).await)
}

async fn subscribe_request(
    client: &Client<HttpConnector>,
    to_addr: &String,
    my_addr: &String,
    begin: usize,
    end: usize,
) {
    let req = Request::builder()
        .method(Method::POST)
        .uri(&format!("http://{to_addr}/bootstrap"))
        .body(Body::from(
            serde_json::to_string(&(format!("http://{my_addr}/insert"), begin, end)).unwrap(),
        ))
        .unwrap();
    client.request(req).await.unwrap();
}
