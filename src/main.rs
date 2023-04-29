use anyhow::Result;
use futures::Future;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn};
use hyper::{Method, Request, Response};
use monoio::net::TcpListener;
use monoio_compat::TcpStreamCompat;
use std::env;
use std::net::SocketAddr;

pub(crate) async fn serve_http<S, F, A>(addr: A, mut service: S) -> Result<()>
where
    S: FnMut(Request<hyper::body::Incoming>, SocketAddr) -> F + 'static + Copy,
    F: Future<Output = Result<Response<Full<Bytes>>>> + 'static,
    A: Into<SocketAddr>,
{
    let listener = TcpListener::bind(addr.into())?;
    loop {
        let (stream, addr) = listener.accept().await?;
        monoio::spawn(http1::Builder::new().serve_connection(
            TcpStreamCompat::new(stream),
            service_fn(move |req| async move { service(req, addr).await }),
        ));
    }
}

async fn hyper_handler(
    req: Request<hyper::body::Incoming>,
    addr: SocketAddr,
) -> Result<Response<Full<Bytes>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let mut resp = Response::builder().status(200);
            let real_ip_header = req.headers().get("x-real-ip").cloned();
            if let Some(server) = env::var_os("SERVER") {
                resp = resp.header("server", server.into_string().unwrap());
            }
            if let Some(real_ip) = real_ip_header {
                let resp = resp
                    .body(Full::new(Bytes::from(
                        real_ip.to_str().unwrap().to_string(),
                    )))
                    .unwrap();
                Ok(resp)
            } else {
                let resp = resp
                    .body(Full::new(Bytes::from(addr.ip().to_string())))
                    .unwrap();
                Ok(resp)
            }
        }
        _ => {
            let resp = Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap();
            Ok(resp)
        }
    }
}

#[monoio::main]
async fn main() {
    let _ = serve_http(([0, 0, 0, 0], 8080), hyper_handler).await;
}
