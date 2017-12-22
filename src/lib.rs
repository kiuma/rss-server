//! This crate exposes an Hyper HTTP server with multithreading capabilities.
//! It also defines common traits for services and configurables.

#![feature(proc_macro, conservative_impl_trait, generators, associated_type_defaults)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

extern crate futures;
extern crate tokio_pool;
extern crate tokio_core;

extern crate hyper;


mod errors;
pub use errors::RssError;

mod config;
pub use config::RssConfigurable;

mod server;
pub use server::{RssHttpServer, DefaultRssHttpServer, RssService, ResponseFuture};

pub mod services;
