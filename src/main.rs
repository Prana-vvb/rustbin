use bytes::BufMut;
use futures::{StreamExt, TryStreamExt};
use local_ip_addr::get_local_ip_address;
use short_uuid::ShortUuid;
use std::fs::{self, File};
use std::io::Read;
use std::net::Ipv4Addr;
use warp::reply::Response;
use warp::{Filter, Rejection, Reply};

const SITE: &str = "http://localhost:8080";
const MAX_SIZE: u64 = 1024 * 1024;

#[tokio::main]
async fn main() {
    let _ = fs::create_dir_all("./data");

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
    match get_local_ip_address() {
        Ok(addr) => {
            let ip: Result<Vec<u8>, _> = addr
                .split('.')
                .map(|segment| segment.parse::<u8>())
                .collect();

            match ip {
                Ok(ip_parts) => {
                    let ip_array = [ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3]];
                    let ip = Ipv4Addr::from(ip_array);
                    println!("Pastebin running at {:?}:{}", ip, port);
                    warp::serve(routes).run((ip, port)).await;
                }
                Err(e) => println!("Failed to parse IP segments: {}", e),
            }
        }
        Err(e) => println!("Error: {}", e),
    };
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

        let file_path = format!("data/{}", ShortUuid::generate());
        tokio::fs::write(&file_path, value).await.map_err(|e| {
            eprintln!("{}", e);
            warp::reject::reject()
        })?;
        out = format!("Created file: {}\n", file_path);
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
