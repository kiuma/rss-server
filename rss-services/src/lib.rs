#![feature(proc_macro, conservative_impl_trait, generators, associated_type_defaults)]

#[macro_use]
extern crate route;

extern crate futures_await as futures;
extern crate tokio_core;
extern crate hyper_staticfile;
extern crate rss_engine;
extern crate hyper;

mod service;
mod static_router;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
