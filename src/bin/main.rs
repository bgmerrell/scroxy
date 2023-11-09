use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use ::clap::Parser;
use ::futures::future::{BoxFuture, FutureExt};
use ::http_body_util::BodyExt;
use ::http_body_util::{combinators::BoxBody, Empty};
use ::hyper::body::{Bytes, Incoming as IncomingBody};
use ::hyper::client::conn::http1::Builder;
use ::hyper::server::conn::http1;
use ::hyper::service::Service;
use ::hyper::{Request, Response, StatusCode};
use ::hyper_util::rt::TokioIo;
use ::log::{debug, error, info};
use ::parking_lot::RwLock;
use ::tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Directory where CDB files are stored
    cdb_dir: PathBuf,
}

// TODO: Implement me
struct Database {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ::env_logger::init();
    let args = Args::parse();
    debug!("args: {:?}", args);
    let addr = SocketAddr::from(([0, 0, 0, 0], 9999));
    let db = Arc::new(RwLock::new(Database {}));

    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let db = db.clone();
        let scroxy = Scroxy::new(db);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, scroxy)
                .with_upgrades()
                .await
            {
                info!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

struct Scroxy {
    db: Arc<RwLock<Database>>,
}

impl Service<Request<IncomingBody>> for Scroxy {
    type Error = hyper::Error;
    type Response = Response<BoxBody<Bytes, Self::Error>>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<IncomingBody>) -> Self::Future {
        async move {
            debug!("incoming req: {:?}", req);
            let host = match req.headers().get("Host") {
                Some(h) => match h.to_str() {
                    Ok(h) => h,
                    Err(e) => {
                        debug!("error reading Host header: {e}");
                        return Ok(Self::bad_request());
                    }
                },
                None => return Ok(Self::bad_request()),
            };

            let port = 8000u16;
            let addr = format!("{}:{}", host, port);
            debug!("outgoing addr: {:?}", addr);

            let stream = TcpStream::connect(addr).await.unwrap();
            let io = TokioIo::new(stream);

            let (mut sender, conn) = Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .handshake(io)
                .await
                .unwrap();
            tokio::task::spawn(async move {
                if let Err(err) = conn.await {
                    error!("Connection failed: {:?}", err);
                }
            });

            debug!("outgoing req: {:?}", req);
            let resp = sender.send_request(req).await.unwrap();

            let (parts, body) = resp.into_parts();
            let resp = Response::from_parts(parts, body.boxed());

            Ok(resp)
        }
        .boxed()
    }
}

impl Scroxy {
    fn new(db: Arc<RwLock<Database>>) -> Self {
        Scroxy { db }
    }

    fn empty() -> BoxBody<Bytes, hyper::Error> {
        Empty::<Bytes>::new()
            .map_err(|never| match never {})
            .boxed()
    }

    fn bad_request() -> Response<BoxBody<Bytes, hyper::Error>> {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Self::empty())
            .unwrap()
    }
}
