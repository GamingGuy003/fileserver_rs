use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

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
        )
        .add_arg(
            "r",
            "root",
            "Sets the root folder for files. Default: .",
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

    let root = if let Some(pa) = parser.get_parsed_argument_long("root") {
        info!("Trying to use supplied value for root...");
        pa.value.unwrap_or(".".to_owned())
    } else {
        info!("Using default root folder where instance has been launched...");
        ".".to_owned()
    };

    info!("Starting server on {addr}:{port}, serving '{}'...", root);

    HttpServer::new(
        addr,
        port,
        4,
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
        Box::new(move |request: &HttpRequest| {
            let uri = match request.get_route_param(":uri*".to_owned()) {
                Some(uri) => uri,
                None => {
                    warn!("URI not set");
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
                }
            };
            let root_path = Path::new(&root);
            let uri_path = Path::new(&uri);
            let rooted_path = root_path.join(uri_path);

            debug!("Trying to handle path {}", uri_path.display());
            match rooted_path {
                _ if rooted_path.is_dir() => {
                    handle_folder(uri_path.to_path_buf(), root_path.to_path_buf())
                }
                _ if rooted_path.is_file() => {
                    handle_file(uri_path.to_path_buf(), root_path.to_path_buf())
                }
                _ => {
                    warn!("File {} not found", rooted_path.display());
                    HttpResponse::new(
                        "1.1".to_owned(),
                        HttpStatus::NotFound,
                        None,
                        Some(HttpData::Bytes(
                            format!("File not found. Request was:<br>{:#?}", request)
                                .as_bytes()
                                .to_vec(),
                        )),
                    )
                }
            }
        }),
    )
    .run_loop()
    .expect("Failed to handle connection");
}

pub fn handle_file(path: PathBuf, root: PathBuf) -> HttpResponse {
    let path = root.join(path);
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

pub fn handle_folder(path: PathBuf, root: PathBuf) -> HttpResponse {
    let rooted_path = root.join(&path);
    let files = match fs::read_dir(&rooted_path) {
        Ok(file) => file,
        Err(err) => {
            warn!("Could not open file: {err}");
            return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None);
        }
    };

    let href_up = format!(
        "/{}",
        path.parent().unwrap_or(Path::new(".")).to_string_lossy()
    )
    .replace("//", "/");

    let mut entries = vec![format!("<a href=\"{href_up}\">..</a>",)];

    entries.append(
        &mut files
            .map_while(Result::ok)
            .map(|entry| {
                let binding = entry
                    .path();
                let entry_string = binding
                    .strip_prefix(&root)
                    .unwrap_or(Path::new("."))
                    .display();
                let href_file = format!("/{entry_string}",).replace("//", "/");
                format!(
                    "<a href=\"{href_file}\">{}</a>",
                    entry.file_name().to_string_lossy()
                )
            })
            .collect::<Vec<String>>(),
    );
    entries.sort();

    debug!("Sending {} folder entries...", entries.len());
    HttpResponse {
        data: Some(HttpData::Bytes(entries.join("<br>").as_bytes().to_vec())),
        ..Default::default()
    }
}
