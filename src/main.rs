use bytes::BufMut;
use futures::{StreamExt, TryStreamExt};
use local_ip_addr::get_local_ip_address;
use short_uuid::ShortUuid;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

const SITE: &str = "http://0.0.0.0:8080";
const MAX_SIZE: u64 = 1024 * 1024;

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all("./data").await;

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

    let cors = warp::cors()
        .allow_origin(SITE)
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

    let port = 8080;
    if let Ok(ip_str) = get_local_ip_address() {
        if let Ok(ip) = ip_str.parse::<std::net::IpAddr>() {
            println!("Pastebin running at {}:{}", ip, port);
            warp::serve(routes).run((ip, port)).await;
        }
    }
}

async fn handle_upload(data: warp::multipart::FormData) -> Result<impl Reply, Rejection> {
    let mut parts = data;
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

        let file_path = format!("data/{}", ShortUuid::generate());
        tokio::fs::write(&file_path, value).await.map_err(|e| {
            eprintln!("{}", e);
            warp::reject::reject()
        })?;
        out.push_str(&format!("Created file: {}\n", file_path));
    }

    Ok(out)
}

async fn handle_disp(id: String) -> Result<impl Reply, Rejection> {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        eprintln!("Security Warning: Invalid ID format attempted: {}", id);
        return Err(warp::reject::not_found()); 
    }
    let file_path = format!("data/{}", id);

    match File::open(&file_path).await {
        Ok(mut file) => {
            let mut data = Vec::new();
            match file.read_to_end(&mut data).await {
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
