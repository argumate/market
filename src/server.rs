use std::str;
use std::sync;
use std::thread;
use failure::{err_msg, Error};

use futures;
use futures::future::Future;
use futures::sync::oneshot;
use futures::stream::Stream;

use hyper;
use hyper::{Method, StatusCode};
//use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use serde_json;

use market;
use market::Market;

struct Server {
    channel: sync::mpsc::Sender<
        (
            market::msgs::Request,
            futures::sync::oneshot::Sender<market::msgs::Response>,
        ),
    >,
}

#[derive(Debug)]
enum AppError {
    Canceled, // FIXME
    Hyper(hyper::error::Error),
    Json(serde_json::Error),
    Utf8(str::Utf8Error),
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                let mut response = Response::new();
                response.set_body("Try POSTing JSON to /request");
                Box::new(futures::future::ok(response))
            }
            (&Method::Post, "/request") => {
                let tx = self.channel.clone();
                Box::new(
                    req.body()
                        .concat2()
                        .map_err(|e| AppError::Hyper(e))
                        .and_then(|b| {
                            let req_str = match str::from_utf8(&b) {
                                Ok(req_str) => req_str,
                                Err(utf8_error) => return Err(AppError::Utf8(utf8_error)),
                            };
                            serde_json::from_str::<market::msgs::Request>(req_str)
                                .map_err(|e| AppError::Json(e))
                        })
                        .map(move |market_req| {
                            let (reply, on_reply) = oneshot::channel::<market::msgs::Response>();
                            let msg = (market_req, reply);
                            futures::future::result(tx.send(msg))
                                .map_err(|_| AppError::Canceled)
                                .and_then(|_| {
                                    on_reply.map_err(|_| AppError::Canceled).and_then(
                                        |market_reply| {
                                            serde_json::to_string(&market_reply)
                                                .map_err(|e| AppError::Json(e))
                                        },
                                    )
                                })
                        })
                        .flatten()
                        .then(|res| match res {
                            Ok(reply_json) => Ok(Response::new().with_body(reply_json)),
                            Err(err) => match err {
                                AppError::Canceled => Ok(Response::new()
                                    .with_status(hyper::StatusCode::InternalServerError)
                                    .with_body("internal server error")),
                                AppError::Hyper(err0) => Err(err0),
                                AppError::Json(err0) => Ok(Response::new()
                                    .with_status(hyper::StatusCode::BadRequest)
                                    .with_body(format!("{:?}", err0))),
                                AppError::Utf8(err0) => Ok(Response::new()
                                    .with_status(hyper::StatusCode::BadRequest)
                                    .with_body(format!("{:?}", err0))),
                            },
                        }),
                )
            }
            _ => {
                let mut response = Response::new();
                response.set_status(StatusCode::NotFound);
                Box::new(futures::future::ok(response))
            }
        }
    }
}

fn work_thread(
    mut market: Market,
    rx: sync::mpsc::Receiver<
        (
            market::msgs::Request,
            futures::sync::oneshot::Sender<market::msgs::Response>,
        ),
    >,
) -> Result<(), Error> {
    loop {
        let (req, reply) = rx.recv()?;
        let response = market.do_request(req)?;
        match reply.send(response) {
            Ok(()) => {}
            Err(_) => break,
        }
    }
    Ok(())
}

pub fn run_server(market: Market, addr_str: &str) -> Result<(), Error> {
    let (tx, rx) = sync::mpsc::channel();
    let thread_handle = thread::spawn(move || work_thread(market, rx));
    let (_shutdown, on_shutdown) = oneshot::channel::<()>();
    let addr = addr_str.parse()?;
    Http::new()
        .bind(&addr, move || {
            Ok(Server {
                channel: tx.clone(),
            })
        })?
        .run_until(on_shutdown.map_err(|_| ()))?;

    match thread_handle.join() {
        Ok(res) => res,
        Err(_) => Err(err_msg("could not join thread")),
    }

    // FIXME shutdown server
}

// vi: ts=8 sts=4 et
