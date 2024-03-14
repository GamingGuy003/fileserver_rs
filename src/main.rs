use std::{fs::File, path::Path};

use cali::parser::Parser;
use http_serv::{http_server::server::HttpServer, HttpData, HttpRequest, HttpResponse, HttpStatus};
use log::{debug, info, warn};

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut parser = Parser::new()
        .add_arg(
            "p",
            "port",
            "Sets the server port. Default: 8080",
            true,
            true,
        )
        .add_arg(
            "a",
            "addr",
            "Sets the bound address. Default: 127.0.0.1",
            true,
            true,
        );

    parser.parse().expect("Failed to parse arguments");

    let addr = if let Some(pa) = parser.get_parsed_argument_long("addr") {
        info!("Trying to use supplied value for addr...");
        pa.value.unwrap_or("127.0.0.1".to_owned())
    } else {
        info!("Using default value 127.0.0.1 for addr...");
        "127.0.0.1".to_owned()
    };

    let port = if let Some(pa) = parser.get_parsed_argument_long("port") {
        info!("Trying to use supplied value for port...");
        pa.value.unwrap_or("8080".to_owned())
    } else {
        info!("Using default value 8080 for port...");
        "8080".to_owned()
    };

    info!("Starting server on {addr}:{port}...");

    HttpServer::new(
        addr,
        port,
        Vec::new(),
        Some(Box::new(|request: &HttpRequest| {
            debug!("Got unimplemented request {:?}", request);
            HttpResponse {
                data: Some(HttpData::Bytes(
                    format!("Not implemented. Request was:<br>{:#?}", request)
                        .as_bytes()
                        .to_vec(),
                )),
                ..Default::default()
            }
        })),
    )
    .expect("Failed to create webserver")
    .get(
        "/:uri*".to_owned(),
        Box::new(|request: &HttpRequest| {
            info!(
                "Handling {:?} {}",
                request.http_headers.method, request.http_headers.path
            );
            let filename = match request.get_route_param(":uri*".to_owned()) {
                Some(filename) => filename,
                None => {
                    warn!("Filename not set");
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
                }
            };

            let path = Path::new(&filename);
            let file = match File::open(&filename) {
                Ok(file) => file,
                Err(err) => {
                    warn!("Could not open file: {err}");
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
                }
            };
            let len = match file.metadata() {
                Ok(metadata) => metadata.len() as usize,
                Err(err) => {
                    warn!("Could not get metadat: {err}");
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
                }
            };

            debug!("Sending {} with length {}", path.display(), len);
            HttpResponse {
                data: Some(HttpData::Stream((Box::new(file), len))),
                ..Default::default()
            }
        }),
    )
    .run_loop()
    .expect("Failed to handle connection");
}
