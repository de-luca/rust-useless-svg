#[macro_use]
extern crate lazy_static;

use actix_web::{
    App,
    Error,
    HttpResponse,
    HttpServer,
    middleware,
    Responder,
    Result,
    web,
};
use actix_web::http::header::{
    CACHE_CONTROL,
    CacheControl,
    CacheDirective,
    ETAG,
    EntityTag,
};
use rand::{Rng};
use bit_vec::BitVec;

const GAP: usize = 10;
const WIDTH: usize = 500;
const HEIGHT: usize = 250;

lazy_static! {
    static ref COLS: usize = WIDTH / GAP - 1;
    static ref ROWS: usize = ((HEIGHT / GAP) * 2) - 1;
}

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
    fn hex(&self) -> String {
        format!("#{:x}{:x}{:x}", self.red, self.green, self.blue)
    }
}


async fn index() -> Result<impl Responder, Error> {
    let mut rng = rand::thread_rng();
    let col_colors: Vec<Color> = (0..*COLS).map(|_| Color::random()).collect();

    let mut svg = format!(
        "<svg width=\"{}px\" height=\"{}px\" xmlns=\"http://www.w3.org/2000/svg\">",
        WIDTH, HEIGHT,
    );

    for row in 0..*ROWS {
        let row_color = Color::random();
        let mut bits = BitVec::from_bytes(
            &rng.gen_range(i64::MIN, i64::MAX).to_be_bytes()
        );
        bits.truncate(*COLS);

        let even = row % 2 == 0;
        let mut col = 0;

        for display in bits.iter() {
            if display {
                svg.push_str(format!(
                    "<path d=\"M {} {} L {} {}\" stroke=\"{}\" stroke-width=\"2\" />",
                    col * GAP + if even { GAP } else { 0 },
                    row * GAP + if even { 0 } else { GAP },
                    col * GAP + (GAP * if even { 1 } else { 2 }),
                    row * GAP + (GAP * if even { 2 } else { 1 }),
                    if even { col_colors.get(col).unwrap().hex() } else { row_color.hex() },
                ).as_str());
            }

            col = col + 1;
        }
    }

    svg.push_str("</svg>");

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
