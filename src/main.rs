use futures::{StreamExt, TryStreamExt};
use short_uuid::ShortUuid;
use std::env;
use std::path::Path;
use tokio::fs::File;
use tokio_util::io::{ReaderStream, StreamReader};
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

const MAX_SIZE: u64 = 1024 * 1024;
const DEFAULT_LIMIT: usize = 512;

static UPLOAD_LIMIT: std::sync::LazyLock<tokio::sync::Semaphore> = std::sync::LazyLock::new(|| {
    let limit = env::var("UPLOAD_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(DEFAULT_LIMIT);

    tokio::sync::Semaphore::new(limit)
});
static DOWNLOAD_LIMIT: std::sync::LazyLock<tokio::sync::Semaphore> =
    std::sync::LazyLock::new(|| {
        let limit = env::var("DOWNLOAD_LIMIT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&v| v > 0)
            .unwrap_or(DEFAULT_LIMIT);

        tokio::sync::Semaphore::new(limit)
    });

#[tokio::main]
async fn main() {
    let _ = std::fs::create_dir_all("./data");
    let allowed_origin =
        env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    let upload = warp::path("data")
        .and(warp::post())
        .and(warp::multipart::form().max_length(MAX_SIZE))
        .and_then(handle_upload);

    let disp = warp::get()
        .and(warp::path!("data" / String))
        .and_then(handle_disp);

    let robots_txt = warp::path("robots.txt").map(|| {
        warp::reply::with_header("User-agent: *\nDisallow: /", "Content-Type", "text/plain")
    });

    let cors = warp::cors()
        .allow_origin(allowed_origin.as_str())
        .allow_methods(vec!["GET", "POST", "OPTIONS"])
        .allow_headers(vec![
            "Content-Type",
            "Access-Control-Allow-Headers",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "Origin",
            "Accept",
            "X-Requested-With",
        ]);

    let routes = upload.or(disp).or(robots_txt).with(cors);

    println!(
        "Pastebin running on port {} allowing CORS for {}",
        port, allowed_origin
    );
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn handle_upload(data: warp::multipart::FormData) -> Result<impl Reply, Rejection> {
    let _permit = UPLOAD_LIMIT.acquire().await.unwrap();

    let mut parts = data;
    let mut out = String::new();

    while let Some(Ok(p)) = parts.next().await {
        let mut file_path;
        let mut id;
        let mut file;

        loop {
            id = ShortUuid::generate().to_string();
            file_path = Path::new("data").join(id);

            match tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&file_path)
                .await
            {
                Ok(f) => {
                    file = f;
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    continue;
                }
                Err(e) => {
                    eprintln!("File creation error: {}", e);
                    return Err(warp::reject::reject());
                }
            }
        }

        let io_stream = p.stream().map_err(std::io::Error::other);
        let mut reader = StreamReader::new(io_stream);

        if let Err(e) = tokio::io::copy(&mut reader, &mut file).await {
            eprintln!("File write error: {}", e);
            return Err(warp::reject::reject());
        }

        out.push_str(&format!("Created file: {:?}\n", file_path));
    }

    Ok(out)
}

async fn handle_disp(id: String) -> Result<impl Reply, Rejection> {
    let _permit = DOWNLOAD_LIMIT.acquire().await.unwrap();

    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        eprintln!("Security Warning: Invalid ID format attempted: {}", id);
        return Err(warp::reject::not_found());
    }
    let file_path = Path::new("data").join(id);

    match File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = warp::hyper::Body::wrap_stream(stream);

            Ok(Response::new(body))
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(warp::reject::reject())
        }
    }
}
