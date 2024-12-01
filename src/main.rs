use futures::{StreamExt, TryStreamExt};
use bytes::BufMut;
use warp::reply::Response;
use std::fs::File;
use std::io::Read;
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

    //Let the bots have all your data hehehe
    let robots_txt = warp::path("robots.txt").map(|| {
        warp::reply::with_header("User-agent: *\nDisallow: /", "Content-Type", "text/plain")
    });

    let routes = upload.or(disp).or(robots_txt);

    let port = 8080;
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn handle_upload(data: warp::multipart::FormData) -> Result<impl Reply, Rejection> {
    let mut parts = data.into_stream();
    let mut out = String::new();

    while let Some(Ok(p)) = parts.next().await {
        let value = p
            .stream()
            .try_fold(Vec::new(), |mut vec, data| {
                vec.put(data);
                async move { Ok(vec) }
            })
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                warp::reject::reject()
            })?;

        let file_path = format!("data/{}", Uuid::new_v4());
        tokio::fs::write(&file_path, value).await.map_err(|e| {
            eprintln!("{}", e);
            warp::reject::reject()
        })?;
        out = format!("Created file: {}/{}", SITE, file_path);
    }

    Ok(out)
}

async fn handle_disp(id: String) -> Result<impl Reply, Rejection> {
    let file_path = format!("data/{}", id);

    match File::open(&file_path) {
        Ok(mut file) => {
            let mut data = Vec::new();
            match file.read_to_end(&mut data) {
                Ok(_) => Ok(Response::new(data.into())),
                Err(e) => {
                    eprintln!("{}", e);
                    Err(warp::reject::reject())
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(warp::reject::reject())
        }
    }
}
