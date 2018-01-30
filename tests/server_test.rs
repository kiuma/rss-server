extern crate futures;
extern crate hyper;
extern crate rss_server;
extern crate tokio_core;

use hyper::client::{Client, FutureResponse};
use hyper::StatusCode;
// use hyper::Body;
use tokio_core::reactor::{Core, Handle};
use futures::future::Future;
use futures::prelude::*;
use futures::sync::oneshot::{channel as future_channel, Receiver as FutureReceiver};
use std::rc::Rc;
use std::thread;
use std::sync::mpsc;
use hyper::server::Http;
use hyper::Error;
// use std::str::from_utf8;
use std::vec::Vec;
use futures::future::ok;

mod sample_site;
use sample_site::get_site_service;

// use sample_site;

fn serve(shutdown_rx: FutureReceiver<bool>) -> std::net::SocketAddr {
    let addr = "127.0.0.1:0".parse().unwrap();
    let (addr_tx, addr_rx) = mpsc::channel();
    thread::Builder::new()
        .name(String::from("test-server"))
        .spawn(move || {
            let site_service = Rc::new(get_site_service());
            let srv = Http::new()
                .bind(&addr, move || Ok(Rc::clone(&site_service)))
                .unwrap();
            addr_tx.send(srv.local_addr().unwrap()).unwrap();
            srv.run_until(shutdown_rx.then(|_| Ok(()))).unwrap();
        })
        .unwrap();

    addr_rx.recv().unwrap()
}

fn do_get(handle: &Handle, port: u16, path: &str) -> FutureResponse {
    let client = Client::new(handle);
    let uri = format!("http://localhost:{}/{}", port, path)
        .parse()
        .unwrap();
    client.get(uri)
}

fn test_resource(payload: Vec<(&str, StatusCode, &str)>) {
    let (shutdown_tx, shutdown_rx) = future_channel();
    let (client_tx, client_rx) = mpsc::channel();
    let mut core = Core::new().unwrap();

    let socket_addr = serve(shutdown_rx);
    let hanlde = &core.handle();

    for (page, exp_status, exp_body) in payload {
        core.run(do_get(hanlde, socket_addr.port(), page).and_then(|res| {
            assert_eq!(res.status(), exp_status);

            res.body()
                .fold(Vec::new(), |mut v, chunk| {
                    v.extend(&chunk[..]);
                    ok::<_, Error>(v)
                })
                .map(|chunks| {
                    let s = String::from_utf8(chunks).unwrap();
                    assert_eq!(s, exp_body);
                    client_tx.send(true).unwrap();
                })
        })).unwrap();
    }
    client_rx.recv().unwrap();
    shutdown_tx.send(true).unwrap();
}
//
// fn test_resource(page: &str, exp_status: StatusCode, exp_body: &str) {
//     let (shutdown_tx, shutdown_rx) = future_channel();
//     let (client_tx, client_rx) = mpsc::channel();
//     let mut core = Core::new().unwrap();
//
//     let socket_addr = serve(shutdown_rx);
//     let hanlde = &core.handle();
//     core.run(do_get(hanlde, socket_addr.port(), page).and_then(|res| {
//         assert_eq!(res.status(), exp_status);
//
//         res.body()
//             .fold(Vec::new(), |mut v, chunk| {
//                 v.extend(&chunk[..]);
//                 ok::<_, Error>(v)
//             })
//             .map(|chunks| {
//                 let s = String::from_utf8(chunks).unwrap();
//                 assert_eq!(s, exp_body);
//                 shutdown_tx.send(true).unwrap();
//                 client_tx.send(true).unwrap();
//             })
//     })).unwrap();
//
//     client_rx.recv().unwrap();
// }

#[test]
fn test_page1() {
    test_resource(vec![("page1", StatusCode::Ok, "page1")]);
}

#[test]
fn test_page2() {
    test_resource(vec![("page2", StatusCode::Ok, "page2")]);
}

#[test]
fn test_page3() {
    test_resource(vec![("page3", StatusCode::Ok, "page3")]);
}

#[test]
fn test_page_not_found() {
    let expected = format!("{}", StatusCode::NotFound.as_u16());
    test_resource(vec![
        ("notAValidPage", StatusCode::NotFound, expected.as_str()),
    ]);
}

#[test]
fn test_page_multiple_resources() {
    let expected = format!("{}", StatusCode::NotFound.as_u16());
    test_resource(vec![
        ("page3", StatusCode::Ok, "page3"),
        ("page3", StatusCode::Ok, "page3"),
        ("notAValidPage", StatusCode::NotFound, expected.as_str()),
        ("page1", StatusCode::Ok, "page1"),
    ]);
}
