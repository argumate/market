use std::str;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use failure::{err_msg, Error};

use futures;
use futures::future::Future;
use futures::sync::oneshot;

use serde_json;

use actix;
use actix_web::{App, AsyncResponder, HttpMessage, HttpRequest, HttpResponse, FutureResponse};
use actix_web::error;
use actix_web::server;

use crate::market::{self, Market};

type ResponseFuture = futures::sync::oneshot::Sender<market::msgs::Response>;

struct AppState {
    channel: Arc<Mutex<mpsc::Sender<(AppMsg, ResponseFuture)>>>,
}

enum AppMsg {
    Request(market::msgs::Request),
    //FIXME Shutdown,
}

#[derive(Debug)]
enum AppError {
    Canceled, // FIXME
    Payload(error::PayloadError),
    Json(serde_json::Error),
    Utf8(str::Utf8Error),
}

fn make_error(err: AppError) -> HttpResponse {
    HttpResponse::BadRequest().body(format!("{:?}", err))
}

fn make_ok(str: String) -> HttpResponse {
    HttpResponse::Ok().body(str)
}

fn handle_post(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let tx = req.state().channel.lock().unwrap().clone();
    // req.payload().concat2() gives denial of service on big payloads
    req.body()
        .map_err(|e| AppError::Payload(e))
        .and_then(|b| {
            let req_str = match str::from_utf8(&b) {
                Ok(req_str) => req_str,
                Err(utf8_error) => return Err(AppError::Utf8(utf8_error)),
            };
            serde_json::from_str::<market::msgs::Request>(req_str)
                .map_err(|e| AppError::Json(e))
                .map(|market_req| AppMsg::Request(market_req))
        })
        .map(move |msg| {
            let (reply, on_reply) = oneshot::channel::<market::msgs::Response>();
            futures::future::result(tx.send((msg, reply)))
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
        .then(|r| {
            match r {
                Ok(s) => Ok(make_ok(s)),
                Err(e) => Ok(make_error(e)),
            }
        }).responder()
}

fn work_thread(
    mut market: Market,
    rx: mpsc::Receiver<(AppMsg, ResponseFuture)>,
) -> Result<(), Error> {
    loop {
        let (msg, reply) = rx.recv()?;
        match msg {
            AppMsg::Request(req) => {
                let response = market.do_request(req)?;
                match reply.send(response) {
                    Ok(()) => {},
                    Err(_req) => return Err(err_msg("http thread not responding")),
                }
            },
        }
    }
}

pub fn run_server(market: Market, addr_str: &str) -> Result<(), Error> {
    let sys = actix::System::new("market");

    let (tx, rx) = mpsc::channel();
    let thread_handle = thread::spawn(move || work_thread(market, rx));
    let arc_mutex_tx = Arc::new(Mutex::new(tx));

    let _ = server::new(move || {
        App::with_state(AppState { channel: arc_mutex_tx.clone(), })
            .resource("/", |r| r.post().a(handle_post))
    }).bind(addr_str)?.start();

    let _ = sys.run();

    match thread_handle.join() {
        Ok(res) => res,
        Err(_) => Err(err_msg("could not join thread")),
    }
}

// vi: ts=8 sts=4 et
