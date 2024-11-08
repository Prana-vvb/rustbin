use std::convert::Infallible;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::Filter;

const SITE: &str = "http://localhost:8080";
const MAX_SIZE: usize = 1 * 1024 * 1024;

#[tokio::main]
async fn main() {
    fs::create_dir_all("data").expect("Unable to create data directory");

    let upload = warp::post()
        .and(warp::path::end())
        .and(warp::multipart::form())
        .and_then(handle_upload);

    let disp = warp::get()
        .and(warp::path!("data" / String));
        //.and_then(handle_disp);

    //Why block bots tho
    let robots_txt = warp::path("robots.txt").map(|| {
        warp::reply::with_header("User-agent: *\nDisallow: /", "Content-Type", "text/plain")
    });

    let routes = upload.or(disp).or(robots_txt);

    let port = 8080;
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn handle_upload(data: warp::multipart::FormData) -> Result<impl warp::Reply, Infallible> {
    //if data.len() > MAX_SIZE {
    //    return Ok(warp::reply::with_status(
    //        "File size cannot be > 1 MB",
    //        warp::http::StatusCode::BAD_REQUEST,
    //    ));
    //}

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

    if file.len() == 0 {
        return Ok(warp::reply::with_status(
            "File is empty",
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let id = Uuid::new_v4().to_string();
    let new_file_path = format!("data/{}", id);
    let mut new_file = match File::create(&new_file_path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Couldn't save file",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    Ok(warp::reply::with_status(
        &format!("File pasted at {}/data/{}", SITE, id),
        warp::http::StatusCode::OK,
    ))
}
