use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use clap::Parser;
use http::Method;
use image::{io::Reader, ImageError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    error::Error,
    io::Cursor,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    str::FromStr,
};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[clap(
    name = "lcs-png-diff-server",
    about = "A server for generating diff bitmaps from png files"
)]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "info")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./assets")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    // Support CORS for GET and POST
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(vec![Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/diff", post(diff))
        .merge(SpaRouter::new("/assets", opt.static_dir))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        );

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::UNSPECIFIED)),
        opt.port,
    ));

    log::info!("LCS Diff server listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Unable to start server");
}

#[derive(Deserialize)]
struct Diff {
    before_png: String,
    after_png: String,
}

#[derive(Serialize)]
struct DiffResult {
    result_url: String,
}

async fn diff(Json(payload): Json<Diff>) -> Result<Json<DiffResult>, AppError> {
    let before_png = reqwest::get(payload.before_png).await?;
    let before_png_raw_data = before_png.bytes().await?;
    let before = Reader::new(Cursor::new(before_png_raw_data))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()?;

    let after_png = reqwest::get(payload.after_png).await?;
    let after_png_raw_data = after_png.bytes().await?;
    let after = Reader::new(Cursor::new(after_png_raw_data))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()?;

    let result = lcs_png_diff::diff(&before, &after).unwrap();

    let result_file = format!("{}{}{}", "assets/", Uuid::new_v4(), ".png");

    result.save(&result_file)?;

    log::info!(
        "{}{}{} Generating diff bitmap at {}",
        '\u{1F31F}',
        '\u{1F31F}',
        '\u{1F31F}',
        result_file
    );

    let host_info = env::var("HOST_INFO").unwrap_or("http://localhost:8080/".to_string());

    Ok(Json(DiffResult {
        result_url: format!("{}{}", host_info, result_file),
    }))
}

/// App's top level error type.
enum AppError {
    InputNotFound,
    UnsupportedBitmapFormat,
    UnknownError,
}

impl From<reqwest::Error> for AppError {
    fn from(inner: reqwest::Error) -> Self {
        log::error!("{}{}{} {}", '\u{2757}', '\u{2757}', '\u{2757}', inner);
        AppError::InputNotFound
    }
}

impl From<ImageError> for AppError {
    fn from(inner: ImageError) -> Self {
        log::error!("{}{}{} {}", '\u{2757}', '\u{2757}', '\u{2757}', inner);
        AppError::UnsupportedBitmapFormat
    }
}

impl From<Box<dyn Error>> for AppError {
    fn from(inner: Box<dyn Error>) -> Self {
        log::error!("{}{}{} {}", '\u{2757}', '\u{2757}', '\u{2757}', inner);
        AppError::UnknownError
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InputNotFound => (StatusCode::NOT_FOUND, "Input not found"),
            AppError::UnsupportedBitmapFormat => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Only supports image/png",
            ),
            AppError::UnknownError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!(" signal received, starting graceful shutdown");
}
