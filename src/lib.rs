//! This crate exposes an Hyper HTTP server with multithreading capabilities.
//!
//! It also defines common traits for services and configurables.

//#![feature(proc_macro, conservative_impl_trait, generators, associated_type_defaults)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate futures;
extern crate tokio_core;
extern crate tokio_pool;

extern crate hyper;

mod errors;
pub use errors::{HttpError, RssError};

mod config;
pub use config::RssConfigurable;

mod server;
pub use server::{HttpServer, ResponseFuture, RssHttpServer, RssService};

mod services;
pub use services::{Router, RouterService};
