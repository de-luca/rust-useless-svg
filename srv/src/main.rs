use svg;


use actix_web::{App, Error, HttpResponse, Responder, Result, HttpServer, middleware, web};
use actix_web::http::header::{
    CACHE_CONTROL,
    CacheControl,
    CacheDirective,
    ETAG,
    EntityTag,
};


async fn index() -> Result<impl Responder, Error> {
    let svg: String = svg::svg();
    Ok(
        HttpResponse::Ok()
            .content_type("image/svg+xml")
            .body(&svg)
            .with_header(
                CACHE_CONTROL,
                CacheControl(vec![
                    CacheDirective::NoCache,
                    CacheDirective::MaxAge(0),
                ])
            )
            .with_header(
                ETAG,
                EntityTag::new(
                    false,
                    format!("{:x}", md5::compute(&svg.as_bytes())),
                )
            )

    )
}


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/useless.svg").route(web::get().to(index)))
    })
        .bind("0.0.0.0:7878")?
        .run()
        .await
}
