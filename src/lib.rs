//! TODO Write proper description.
#![feature(conservative_impl_trait)]

extern crate futures;
extern crate futures_spawn;
extern crate futures_threadpool;
extern crate http_box;
extern crate num_cpus;
extern crate tokio_core;
extern crate tokio_pool;

extern crate hyper;

mod errors;
pub use errors::HttpError;

mod services;
pub use services::{ErrorHandler, FormData, Multipart, MultipartData, ResponseFuture, Router,
                   RouterService, RssService};
