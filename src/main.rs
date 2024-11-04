use std::convert::Infallible;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use uuid::Uuid;
use warp::{wrap_fn, Filter};

const SITE: &str = "http://localhost:8080";
const MAX_SIZE: usize = 1024 * 1024;

#[tokio::main]
async fn main() {
    fs::create_dir_all("data").expect("Unable to create data directory");

    let upload = warp::post()
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and_then(handle_upload);

    let disp = warp::get()
        .and(warp::path!("data" / String))
        .and_then(handle_disp);

    let routes = upload.or(disp);

    //What da robot do

    let port = 8080;
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn handle_upload(data: warp::multipart::Form) -> Result<impl warp::Reply, Infallible> {
    if data.len() > MAX_SIZE {
        return Ok(warp::reply::with_status(
            "File size cannot be > 1 MB",
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let parts = data.files().next();
    let file = match parts {
        Some(part) => part,
        None => {
            return Ok(warp::reply::with_status(
                "Failed to get file",
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    };
}
