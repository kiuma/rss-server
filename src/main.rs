#![feature(proc_macro, conservative_impl_trait, generators)]
#[macro_use]
extern crate log;

extern crate rss_routes;
extern crate hyper;
extern crate multipart_async as multipart;
extern crate futures_await as futures;
extern crate tokio_pool;
extern crate tokio_core;

use tokio_pool::TokioPool;

use futures::prelude::*;

use rss_routes::RouterService;

use hyper::server::Http;

mod multipart_test;

use multipart_test::MultipartTest;

use tokio_core::net::TcpListener;

use std::sync::Arc;

fn main() {
    // Create a pool with 4 workers
    let (pool, join) = TokioPool::new(4).expect("Failed to create event loop");
    // Wrap it in an Arc to share it with the listener worker
    let pool = Arc::new(pool);

    let server_address = "127.0.0.1:3000";
    let addr = server_address.parse().unwrap();

    // Clone the pool reference for the listener worker
    let pool_ref = pool.clone();

    //    let http = Http::new();

    //    // Use the first pool worker to listen for connections
    //    pool.next_worker().spawn(move |handle| {
    //        // Bind a TCP listener to our address
    //        let listener = TcpListener::bind(&addr, handle).unwrap();
    //        // Listen for incoming clients
    //        listener.incoming().for_each(move |(socket, addr)| {
    //            pool_ref.next_worker().spawn(move |handle| {
    //                // Do work with a client socket
    ////                let s = RouterService::new(&handle);
    ////                http.bind_connection(&handle, socket, addr, s);
    //                Ok(())
    //            });
    //            Ok(())
    //
    //
    //        });
    //
    ////            .map_err(|err| {
    ////            error!("Error with TcpListener: {:?}", err);
    ////        })
    //    });
    //
    //    join.join();
    // Use the first pool worker to listen for connections
    pool.next_worker().spawn(move |handle| {
        // Bind a TCP listener to our address
        let listener = TcpListener::bind(&addr, handle).unwrap();
        // Listen for incoming clients
        listener
            .incoming()
            .for_each(move |(socket, addr)| {
                pool_ref.next_worker().spawn(move |handle| {
                    let http = Http::new();
                    let s = RouterService::new(&handle);
                    http.bind_connection(&handle, socket, addr, s);
                    // Do work with a client socket
                    Ok(())
                });

                Ok(())
            })
            .map_err(|_| ()) // You might want to log these errors or something
    });
    join.join()
    //    let func = || Ok(Router);
    //    let server = Http::new().bind(&addr,
    //                                  func).unwrap();
    //    println!("server running: http://{}", server_address);
    //    server.run().unwrap();
}
