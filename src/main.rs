use futures::{StreamExt, TryStreamExt};
use std::convert::Infallible;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use warp::{Filter, Rejection, Reply};

const SITE: &str = "http://localhost:8080";
const MAX_SIZE: u64 = 1 * 1024 * 1024;

#[tokio::main]
async fn main() {
    let upload = warp::path("data")
        .and(warp::post())
        .and(warp::multipart::form().max_length(MAX_SIZE))
        .and_then(handle_upload);

    let disp = warp::get()
        .and(warp::path!("data" / String))
        .and_then(handle_disp);

    //Why block bots tho
    let robots_txt = warp::path("robots.txt").map(|| {
        warp::reply::with_header("User-agent: *\nDisallow: /", "Content-Type", "text/plain")
    });

    let routes = upload.or(disp).or(robots_txt);

    let port = 8080;
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn handle_upload(data: warp::multipart::FormData) -> Result<impl warp::Reply, Infallible> {
    let mut parts = data.into_stream();

    while let Some(Ok(p)) = parts.next().await {
        let value = p.stream().try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        }).await
    }
}
