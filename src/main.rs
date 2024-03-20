use std::{fs::{self, File}, path::{Path, PathBuf}};

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
            let uri = match request.get_route_param(":uri*".to_owned()) {
                Some(uri) => uri,
                None => {
                    warn!("URI not set");
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
                }
            };

            let path = Path::new(&uri);
            debug!("Trying to handle path {}", path.display());
            match path {
                _ if path.is_dir() => handle_folder(path.to_path_buf()),
                _ if path.is_file() => handle_file(path.to_path_buf()),
                _ => {
                    warn!("File {} not found", path.display());
                    HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, Some(HttpData::Bytes(
                    format!("File not found. Request was:<br>{:#?}", request)
                        .as_bytes()
                        .to_vec(),
                )))}
            }       
        }),
    )
    .run_loop()
    .expect("Failed to handle connection");
}

pub fn handle_file(path: PathBuf) -> HttpResponse {
    let file = match File::open(&path) {
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

    debug!("Sending file {} with length {}", path.display(), len);
    HttpResponse {
        data: Some(HttpData::Stream((Box::new(file), len))),
        ..Default::default()
    }
}

pub fn handle_folder(path: PathBuf) -> HttpResponse {
    let files = match fs::read_dir(&path) {
        Ok(file) => file,
        Err(err) => {
            warn!("Could not open file: {err}");
            return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
        }
    };

    let mut entries = vec![format!("<a href=\"/{}\">..</a>", path.parent().unwrap_or(Path::new(".")).to_string_lossy())];

    entries.append(&mut files.map_while(Result::ok).map(|entry| {
        format!("<a href=\"/{}\">{}</a>", path.join(entry.path().file_name().unwrap_or_default()).display(), entry.file_name().to_string_lossy())
    }).collect::<Vec<String>>());
    
    debug!("Sending {} folder entries...", entries.len());
    HttpResponse {
        data: Some(HttpData::Bytes(entries.join("<br>").as_bytes().to_vec())),
        ..Default::default()
    }
}
