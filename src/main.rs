use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use http_serv::{http_server::server::HttpServer, HttpData, HttpRequest, HttpResponse, HttpStatus};

fn main() {
    HttpServer::new(
        "127.0.0.1".to_owned(),
        "8080".to_owned(),
        Vec::new(),
        Some(Box::new(|request: HttpRequest| {
            println!("Got {:#?}", request);
            let mut resp = HttpResponse::default();
            resp.data = Some(HttpData::new(format!("{:#?}", request).as_bytes().to_vec()));
            resp
        })),
    )
    .expect("Failed to create webserver")
    .get(
        "/:uri".to_owned(),
        Box::new(|request: HttpRequest| {
            let filename = match request.get_route_param(":uri*".to_owned()) {
                Some(filename) => filename,
                None => {
                    return HttpResponse::new("1.1".to_owned(), HttpStatus::NotFound, None, None)
                }
            };
            let hugo = match File::open(&filename) {
                Ok(file) => file,
                Err(err) => {
                    return HttpResponse::new(
                        "1.1".to_owned(),
                        HttpStatus::NotFound,
                        None,
                        Some(HttpData::new(err.to_string().as_bytes().to_vec())),
                    )
                }
            };

            let bufreader = BufReader::new(hugo);
            let mut lines = Vec::new();
            bufreader
                .lines()
                .map_while(Result::ok)
                .for_each(|line| lines.push(line));

            let mut resp = HttpResponse::default();
            println!("Sending {} with length {}", filename, lines.len());
            resp.data = Some(HttpData::new(
                lines.join("\n").as_bytes().to_vec(),
            ));
            resp
        }),
    )
    .run_loop()
    .expect("Failed to handle connection");
    /*
    .get("/:uri".to_owned(), Box::new(|request| {

    }));
    */
}
