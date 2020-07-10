#[macro_use]
extern crate lazy_static;

use serde::{Serialize};
use actix_web::{App, error, Error, HttpResponse, HttpServer, middleware, Result, web};
use tera::{Context, Tera};
use rand::{Rng};

const GAP: usize = 10;
const WIDTH: usize = 500;
const HEIGHT: usize = 250;

lazy_static! {
    static ref COLS: usize = WIDTH / GAP - 1;
    static ref ROWS: usize = HEIGHT / GAP - 1;
}

#[derive(Serialize, Copy, Clone)]
struct Color {
    red: i16,
    green: i16,
    blue: i16,
}

impl Color {
    fn random() -> Color {
        let mut rng = rand::thread_rng();

        Color {
            red: rng.gen_range(0, 256),
            green: rng.gen_range(0, 256),
            blue: rng.gen_range(0, 256),
        }
    }
}

async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let mut rng = rand::thread_rng();

    let mut matrix: Vec<Vec<(i8, i8)>> = vec![];
    let mut color_v: Vec<Color> = vec![];
    let mut color_h: Vec<Color> = vec![];

    for _r in 0..*ROWS {
        color_h.push(Color::random());
        let mut row: Vec<(i8, i8)> = vec![];
        for _c in 0..*COLS {
            color_v.push(Color::random());
            row.push((rng.gen_range(0, 2), rng.gen_range(0, 2)));
        }
        matrix.push(row);
    }

    let mut ctx = Context::new();
    ctx.insert("GAP", &GAP);
    ctx.insert("WIDTH", &WIDTH);
    ctx.insert("HEIGHT", &HEIGHT);
    ctx.insert("matrix", &matrix);
    ctx.insert("color_v", &color_v);
    ctx.insert("color_h", &color_h);

    let s = tmpl.render("useless.svg", &ctx)
        .map_err(|e| {
            dbg!(e);
            error::ErrorInternalServerError("Template error")
        })?;

    Ok(HttpResponse::Ok().content_type("image/svg+xml").body(s))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        let tera =
            Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .data(tera)
            .wrap(middleware::Logger::default())
            .service(web::resource("/useless.svg").route(web::get().to(index)))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
