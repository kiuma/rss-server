#![feature(proc_macro, conservative_impl_trait, generators, associated_type_defaults)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate futures_await as futures;
extern crate tokio_pool;
extern crate tokio_core;

extern crate hyper;

use tokio_pool::TokioPool;

use futures::prelude::*;

use tokio_core::net::TcpListener;

use std::sync::Arc;

use hyper::Error as HyperError;
use hyper::server::{Http, Request as HyperRequest, Response as HyperResponse, Service as HyperService};

pub type ResponseFuture = Box<Future<Item=HyperResponse, Error=HyperError>>;

mod errors;
pub use errors::RssError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    bind_address: String,
    bind_port: u16,
    num_workers: usize,
    static_paths: Vec<String>,
}

pub type RssService = HyperService<
    Request = HyperRequest,
    Response = HyperResponse,
    Error = HyperError,
    Future = ResponseFuture> + Send + Sync;

pub struct DefaultRssHttpServer {
    _config: Config,
    _service: Arc<RssService>,
}

pub trait RssHttpServer {
    type Err;
    type Item;
    fn new(config: Config, sevice: Box<RssService>) -> Self::Item;
    fn start(&self) -> Result<(), Self::Err>;
}

impl RssHttpServer for DefaultRssHttpServer {
    type Err = RssError;
    type Item = DefaultRssHttpServer;
    fn new(config: Config, service: Box<RssService>) -> DefaultRssHttpServer {
        let inner_service = Arc::new(service);
        DefaultRssHttpServer { _config: config, _service: inner_service }
    }

    fn start(&self) -> Result<(), Self::Err> {
        // Create a pool with 4 workers
        let (pool, join) =
            TokioPool::new(self._config.num_workers).expect("Failed to create event loop");
        // Wrap it in an Arc to share it with the listener worker
        let pool = Arc::new(pool);

        let server_address = format!("{}:{}", self._config.bind_address, self._config.bind_port);
        let addr = server_address.parse().unwrap();

        // Clone the pool reference for the listener worker
        let pool_ref = pool.clone();


        let service = self._service.clone();
        pool.next_worker().spawn(move |handle| {
            // Bind a TCP listener to our address
            let listener = TcpListener::bind(&addr, handle).unwrap();
            // Listen for incoming clients
            listener
                .incoming()
                .for_each(move |(socket, addr)| {
                    let inner_service = service.clone();
                    pool_ref.next_worker().spawn(move |handle| {
                        let http = Http::new();
                        http.bind_connection(&handle, socket, addr, inner_service);
                        // Do work with a client socket
                        Ok(())
                    });

                    Ok(())
                })
                .map_err(|err| error!("{}", err)) // You might want to log these errors or something
        });

        join.join();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_config() {
        let config = Config {
            bind_address: "127.0.0.1".to_string(),
            bind_port: 3000,
            num_workers: 4,
            static_paths: vec![
                "/tmp".to_string(),
                "/home/kiuma/tmp".to_string(),
                "/home/kiuma/xtra".to_string(),
            ],
        };
        let serialized = serde_json::to_string(&config).unwrap();

        let des_config: Config = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.bind_address, des_config.bind_address);
        assert_eq!(config.bind_port, des_config.bind_port);
        assert_eq!(config.num_workers, des_config.num_workers);
        assert_eq!(config.static_paths, des_config.static_paths);
    }
}
