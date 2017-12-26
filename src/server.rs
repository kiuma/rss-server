use futures::prelude::*;

use tokio_core::net::TcpListener;

use std::sync::Arc;

use hyper::Error as HyperError;
use hyper::server::{Http, Request as HyperRequest, Response as HyperResponse,
                    Service as HyperService};

use config::RssConfigurable;

use errors::RssError;
use tokio_pool::TokioPool;

use std::fs::File;
use std::io::prelude::*;

use std::path::PathBuf;

use toml;


pub type ResponseFuture = Box<Future<Item = HyperResponse, Error = HyperError>>;

pub type RssService = HyperService<
    Request = HyperRequest,
    Response = HyperResponse,
    Error = HyperError,
    Future = ResponseFuture,
>
                          + Send
                          + Sync;

#[derive(Serialize, Deserialize, Debug)]
struct RssServerConfig {
    pub bind_address: String,
    pub bind_port: u16,
    pub num_workers: usize,
}

///Default implementor of trait [RssHttpServer](trait.RssHttpServer.html)
pub struct DefaultRssHttpServer {
    _config: RssServerConfig,
    service: Arc<RssService>,
}

struct DefaultRssHttpConfigurator {
    path: PathBuf,
}


/// This tarit defines an RSS HTTP server.
/// An RSS HTTP server is a multithreaduing and async/io web server based on [Hyper](https://hyper.rs/) and [futures](https://docs.rs/futures/0.1.17/futures/).
///
/// Through a routing system it drivers the business logic for serving pages and handling error
/// messages.
pub trait RssHttpServer {
    type Err;
    type Item;
    /// Creates a new server defining the configuration path (used by services implementing [RssConfigurable](trait.RssConfigurable.html))
    /// and the root service, the entry point to handle the request dispatching logic.
    ///
    fn new(config_path: PathBuf, service: Box<RssService>) -> Self::Item;
    /// Starts the server and begins serving requests
    fn start(&self) -> Result<(), Self::Err>;
}

///Server default configuration, converted using serde. This constant is used when no "http-server.toml"
/// is found in the server config_path.
pub const HTTP_SERVER_CONFIG_STR: &str = r#"
# HTTP server configuration

bind_address = "127.0.0.1"
bind_port = 8080
num_workers = 4
"#;

impl DefaultRssHttpServer {}

impl DefaultRssHttpConfigurator {
    pub(crate) fn get_conf_filename(path: PathBuf) -> PathBuf {
        let mut filename = PathBuf::new();
        filename.push(path.as_path());
        filename.push("http-server.toml");
        filename
    }
    fn save(&self) -> Result<String, RssError> {
        let mut file = File::create(Self::get_conf_filename(self.path.clone())).unwrap();

        file.write_all(HTTP_SERVER_CONFIG_STR.as_bytes())?;
        Ok(String::from(HTTP_SERVER_CONFIG_STR))
    }
}

impl RssConfigurable for DefaultRssHttpConfigurator {
    fn load(&self) -> Result<String, RssError> {
        let file = File::open(Self::get_conf_filename(self.path.clone()));

        match file {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                Ok(contents)
            }
            Err(_) => self.save(),
        }
    }
}

impl RssHttpServer for DefaultRssHttpServer {
    type Err = RssError;
    type Item = DefaultRssHttpServer;
    fn new(config_path: PathBuf, service: Box<RssService>) -> DefaultRssHttpServer {
        let config = DefaultRssHttpConfigurator { path: config_path };
        let content = config.load().unwrap();
        let server_config: RssServerConfig = toml::from_str(content.as_str()).unwrap();
        DefaultRssHttpServer {
            _config: server_config,
            service: Arc::new(service),
        }
    }

    fn start(&self) -> Result<(), Self::Err> {
        //_service: Arc<RssService>,
        // Create a pool with 4 workers
        let (pool, join) =
            TokioPool::new(self._config.num_workers).expect("Failed to create event loop");
        // Wrap it in an Arc to share it with the listener worker
        let pool = Arc::new(pool);

        let server_address = format!("{}:{}", self._config.bind_address, self._config.bind_port);
        let addr = server_address.parse().unwrap();

        // Clone the pool reference for the listener worker
        let pool_ref = pool.clone();

        let service = self.service.clone();
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

//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use std::path::PathBuf;
    use std::fs::remove_file;
    use hyper::StatusCode;

    fn get_conf_dir() -> PathBuf {
        [
            env::var("CARGO_MANIFEST_DIR").unwrap().as_str(),
            "tests",
            "out",
        ].iter()
            .collect()
    }


    #[test]
    fn writes_and_load_config() {
        let conf_dir = get_conf_dir();
        let filename = DefaultRssHttpConfigurator::get_conf_filename(conf_dir.clone());
        if filename.exists() {
            remove_file(filename.clone()).unwrap();
        }

        struct ErrorHandler = {};

rss_router!(ErrorHandler, req, {
    //this shouldn't be called
    ok(StatusCode::InternalServerError)
}, {
    ok(Response::new()
                               .with_status(statusCode)
                               .with_header(ContentLength(HTML_ERROR.len() as u64))
                               .with_body(HTML_ERROR))
});


        let root_service = RootService::new([], error_handler)

        let server = DefaultRssHttpServer::new(conf_dir, ErrorHandler);
        assert!(filename.exists(), "{:?} does not exist", filename);
        let config = server._config;
        let expected = "127.0.0.1";
        assert_eq!(
            config.bind_address,
            expected,
            "Expected bind address {}, but got {}",
            expected,
            config.bind_address
        );
        let expected = 8080;
        assert_eq!(
            config.bind_port,
            expected,
            "Expected bind port {}, but got {}",
            expected,
            config.bind_port
        );
        let expected = 4;
        assert_eq!(
            config.num_workers,
            expected,
            "Expected {} workers, but got {}",
            expected,
            config.num_workers
        );
    }
}
