use svg;
use http::{StatusCode};
use now_lambda::{lambda, error::NowError, IntoResponse, Request, Response};
use std::error::Error;


fn index(_: Request) -> Result<impl IntoResponse, NowError> {
    let svg: String = svg::svg();
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/svg+xml")
        .header("Cache-Control", "no-cache, max-age=0")
        .header("Etag", format!("{:x}", md5::compute(&svg.as_bytes())))
        .body(svg)
        .expect("Internal Server Error");

    Ok(response)
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(lambda!(index))
}
