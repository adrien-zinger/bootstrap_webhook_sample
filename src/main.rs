use hyper::{
    body::Bytes,
    service::{make_service_fn, service_fn},
    Method, StatusCode,
};
use hyper::{Body, Request, Response, Server};
use serde::Deserialize;
use shared_db::{subscribe, EntryModif, SharedDB, Subscriber};
use std::net::SocketAddr;
use std::{convert::Infallible, time::Duration};
mod shared_db;

pub const BOOTSTRAP_SEND_PERIOD: Duration = Duration::from_secs(1);
pub const MAX_CHUNK_SIZE: usize = 20;

async fn services_impl(req: Request<Body>, db: SharedDB) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/insert") => {
            let modifs = deserialize::<Vec<EntryModif>>(&to_bytes(req.into_body()).await);
            for modif in modifs.iter() {
                match modif {
                    EntryModif::Delete(key) => db.remove(key).await,
                    EntryModif::Update((key, value)) => db.update(key, value).await,
                };
            }
            Ok(Response::new(Body::default()))
        }
        (&Method::POST, "/bootstrap") => {
            let (addr, begin, end) =
                deserialize::<(String, usize, usize)>(&to_bytes(req.into_body()).await);
            spawn_bootstapper_sender(addr, begin, end, db.clone()).await;
            Ok(Response::new(Body::default()))
        }
        (&Method::GET, "/size") => Ok(Response::new(Body::from(format!("{}", db.len().await)))),
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

/// Add a subscriber to my db that will stream it in real time and start the
/// _"bootstrap thread"_ if not already done
pub async fn spawn_bootstapper_sender(addr: String, begin: usize, end: usize, db: SharedDB) {
    db.add_subscriber(Subscriber::new(addr, begin, end)).await;

    static SPAWN_ONCE: std::sync::Once = std::sync::Once::new();
    SPAWN_ONCE.call_once(|| {
        tokio::spawn(async move {
            loop {
                let f = async {
                    db.send_chunks().await;
                    tokio::time::sleep(BOOTSTRAP_SEND_PERIOD).await;
                };
                tokio::select! {
                    _ = f => continue,
                    // cancel and return if ctrl-c
                    _ = tokio::signal::ctrl_c() => return
                };
            }
        });
    });
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn parse_input() -> (String, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        (args[1].clone(), None)
    } else if args.len() == 3 {
        (args[1].clone(), Some(args[2].clone()))
    } else {
        println!("error usage:\ncargo run -- {{port}} {{optional bootstrap port}}");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let (port, bootstrap_port) = parse_input();
    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse::<u16>().unwrap()));
    let shared_database = SharedDB::default();
    let db = shared_database.clone();
    let make_svc = make_service_fn(move |_| {
        let db = db.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                services_impl(req, db.clone())
            }))
        }
    });
    println!("Start bootstrapable server on addr {}", addr);
    println!("Example of posts:");
    println!(
        "Example of /insert body: {}",
        serde_json::to_string(&EntryModif::Update((
            "key".to_string(),
            "value".to_string()
        )))
        .unwrap()
    );
    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    if let Some(p) = bootstrap_port {
        subscribe(format!("127.0.0.1:{p}"), format!("127.0.0.1:{}", port)).await;
    }
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
    println!("Shudown... dump entries:\n");
    shared_database.dump().await;
}

async fn to_bytes(body: Body) -> Bytes {
    hyper::body::to_bytes(body).await.unwrap()
}

fn deserialize<'a, T: Deserialize<'a>>(body_bytes: &'a Bytes) -> T {
    serde_json::from_slice(body_bytes).unwrap()
}
