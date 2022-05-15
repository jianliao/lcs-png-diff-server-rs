use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use clap::Parser;
use image::{io::Reader, DynamicImage, ImageError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    error::Error,
    io::Cursor,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    str::FromStr,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[clap(
    name = "lcs-diff-server",
    about = "A server for generating diff bitmap from png files"
)]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
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

    let app = Router::new()
        .route("/api/diff", post(diff))
        .merge(SpaRouter::new("/assets", opt.static_dir))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
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
    let before_png: reqwest::Response = reqwest::get(payload.before_png).await?;
    let before_png_raw_data = before_png.bytes().await?;
    let before: DynamicImage = Reader::new(Cursor::new(before_png_raw_data))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()?;

    let after_png: reqwest::Response = reqwest::get(payload.after_png).await?;
    let after_png_raw_data = after_png.bytes().await?;
    let after: DynamicImage = Reader::new(Cursor::new(after_png_raw_data))
        .with_guessed_format()
        .expect("Cursor io never fails")
        .decode()?;

    let result: DynamicImage = lcs_png_diff::diff(&before, &after).unwrap();

    let result_file: String = format!("{}{}{}", "assets/", Uuid::new_v4(), ".png");
    result.save(&result_file)?;

    log::info!("Generate diff result at {}", result_file);

    Ok(Json(DiffResult {
        result_url: format!("{}{}", "http://localhost:8080/", result_file),
    }))
}

/// App's top level error type.
enum AppError {
    InputNotFound,
    UnsupportedBitmapFormat,
    UnknownError,
}

impl From<reqwest::Error> for AppError {
    fn from(_: reqwest::Error) -> Self {
        AppError::InputNotFound
    }
}

impl From<ImageError> for AppError {
    fn from(_: ImageError) -> Self {
        AppError::UnsupportedBitmapFormat
    }
}

impl From<Box<dyn Error>> for AppError {
    fn from(_: Box<dyn Error>) -> Self {
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
            AppError::UnknownError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unknown internal server error",
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
