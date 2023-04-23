#![deny(warnings)]

use std::net::SocketAddr;

use ::bytes::Bytes;
use ::http_body_util::{combinators::BoxBody, BodyExt, Empty};
use ::hyper::client::conn::http1::Builder;
use ::hyper::server::conn::http1;
use ::hyper::service::service_fn;
use ::hyper::{Request, Response, StatusCode};
use ::log::{debug, error, info};

use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 9999));

    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(stream, service_fn(scroxy))
                .with_upgrades()
                .await
            {
                info!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn scroxy(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    debug!("incoming req: {:?}", req);

    let host = match req.headers().get("Host") {
        Some(h) => match h.to_str() {
            Ok(h) => h,
            Err(e) => {
                debug!("error reading Host header: {e}");
                return bad_request();
            }
        },
        None => return bad_request(),
    };

    let port = req.uri().port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await.unwrap();

    let (mut sender, conn) = Builder::new()
        .preserve_header_case(true)
        .title_case_headers(true)
        .handshake(stream)
        .await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed: {:?}", err);
        }
    });

    debug!("outgoing req: {:?}", req);
    let resp = sender.send_request(req).await?;
    Ok(resp.map(|b| b.boxed()))
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn bad_request() -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    return Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(empty())
        .unwrap());
}
