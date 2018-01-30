use futures::prelude::*;

use std::rc::Rc;

use hyper::Error as HyperError;
use hyper::server::{Http, Request as HyperRequest, Response as HyperResponse,
                    Service as HyperService};

use config::RssConfigurable;

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
>;

#[derive(Serialize, Deserialize, Debug)]
struct RssServerConfig {
    pub bind_address: String,
    pub bind_port: u16,
    pub num_workers: usize,
}

///Default implementor of trait [`HttpServer`](trait.HttpServer.html)
pub struct RssHttpServer {
    _config: RssServerConfig,
    _http: Http,
}

struct DefaultRssHttpConfigurator {
    path: PathBuf,
}

/// Server default configuration, converted using serde. This constant is used when no "http-server.toml"
/// is found in the server `config_path`, this a new toml file with this content is generated.
pub const HTTP_SERVER_CONFIG_STR: &str = r#"
# HTTP server configuration

bind_address = "127.0.0.1"
bind_port = 8080
num_workers = 4
"#;

impl RssHttpServer {
    pub fn new(config_path: PathBuf) -> RssHttpServer {
        let config = DefaultRssHttpConfigurator { path: config_path };
        let content = config.load().unwrap();
        let server_config: RssServerConfig = toml::from_str(content.as_str()).unwrap();
        RssHttpServer {
            _config: server_config,
        }
    }
}

impl DefaultRssHttpConfigurator {
    pub(crate) fn get_conf_filename(path: &PathBuf) -> PathBuf {
        let mut filename = PathBuf::new();
        filename.push(path.as_path());
        filename.push("http-server.toml");
        filename
    }
    fn save(&self) -> Result<String, HyperError> {
        let mut file = File::create(Self::get_conf_filename(&self.path)).unwrap();

        file.write_all(HTTP_SERVER_CONFIG_STR.as_bytes())?;
        Ok(String::from(HTTP_SERVER_CONFIG_STR))
    }
}

impl RssConfigurable for DefaultRssHttpConfigurator {
    fn load(&self) -> Result<String, HyperError> {
        let file = File::open(Self::get_conf_filename(&self.path));

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
//
// impl HttpServer for RssHttpServer {
//     type Err = HyperError;
//     type Item = RssHttpServer;
//
//     fn start(&self, service: Rc<RssService>) -> Result<(), Self::Err> {
//         let server_address = format!("{}:{}", self._config.bind_address, self._config.bind_port);
//         let addr = server_address.parse().unwrap();
//         let server = Http::new().bind(&addr, move || Ok(Rc::clone(&service)))?;
//         server.run()
//     }
// }

//========================== TESTS =====================================================//
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use std::path::PathBuf;
    use std::fs::remove_file;

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
        let filename = DefaultRssHttpConfigurator::get_conf_filename(&conf_dir.clone());
        if filename.exists() {
            remove_file(filename.clone()).unwrap();
        }

        let server = RssHttpServer::new(conf_dir);

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
