#[macro_use]
extern crate lazy_static;

use actix_web::{App, Error, HttpResponse, HttpServer, middleware, Responder, Result, web};
use actix_web::http::header::{
    CACHE_CONTROL,
    CacheControl,
    CacheDirective,
    ETAG,
    EntityTag,
};
use rand::{distributions::{Distribution, Standard}, Rng};
use bit_vec::BitVec;
use rand::prelude::ThreadRng;


const GAP: usize = 10;
const WIDTH: usize = 640;
const HEIGHT: usize = 240;
const TEXT_SIZE: usize = 15;


lazy_static! {
    static ref COLS: usize = WIDTH / GAP;
    static ref ROWS: usize = HEIGHT / GAP - 1;
    static ref COMMIT: String = option_env!("COMMIT").unwrap_or("head").to_string();
}


#[derive(Debug)]
enum ColorPreset {
    Chaos,
    Black,
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
}

impl Distribution<ColorPreset> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ColorPreset {
        match rng.gen_range(0, 8) {
            0 => ColorPreset::Black,
            1 => ColorPreset::Red,
            2 => ColorPreset::Yellow,
            3 => ColorPreset::Green,
            4 => ColorPreset::Cyan,
            5 => ColorPreset::Blue,
            6 => ColorPreset::Purple,
            _ => ColorPreset::Chaos,
        }
    }
}

impl ColorPreset {
    fn hue_range(&self) -> (i16, i16) {
        match self {
            ColorPreset::Chaos => (0, 360),
            ColorPreset::Black => (0, 0),
            ColorPreset::Red => (-30, 30),
            ColorPreset::Yellow => (30, 90),
            ColorPreset::Green => (90, 150),
            ColorPreset::Cyan => (150, 210),
            ColorPreset::Blue => (210, 270),
            ColorPreset::Purple => (330, 390),
        }
    }

    fn saturation_range(&self) -> (i16, i16) {
        match self {
            ColorPreset::Black => (0, 0),
            _ => (0, 100),
        }
    }

    fn name(&self) -> String {
        match self {
            ColorPreset::Chaos => "chaos",
            ColorPreset::Black => "black",
            ColorPreset::Red => "red",
            ColorPreset::Yellow => "yellow",
            ColorPreset::Green => "green",
            ColorPreset::Cyan => "cyan",
            ColorPreset::Blue => "blue",
            ColorPreset::Purple => "purple",
        }.to_string()
    }
}

struct Color {
    hue: i16,
    saturation: i16,
    lightness: i16,
}

impl Color {
    fn random(rng: &mut ThreadRng, preset: &ColorPreset) -> Color {
        let hue_range = preset.hue_range();
        let saturation_range = preset.saturation_range();

        let hue = rng.gen_range(hue_range.0, hue_range.1 + 1);

        Color {
            hue: if hue < 0 { 360 + hue } else { hue },
            saturation: rng.gen_range(saturation_range.0, saturation_range.1 + 1),
            lightness: rng.gen_range(0, 101),
        }
    }

    fn hsl(&self) -> String {
        format!("hsl({},{}%,{}%)", self.hue, self.saturation, self.lightness)
    }
}


async fn index() -> Result<impl Responder, Error> {
    let mut rng = rand::thread_rng();
    let preset: ColorPreset = rand::random();

    let col_colors: Vec<Color> = (0..*COLS).map(|_| Color::random(&mut rng, &preset)).collect();

    let mut svg = format!(
        "<svg viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">",
        WIDTH, HEIGHT + TEXT_SIZE,
    );

    for row in 0..*ROWS {
        let row_color = Color::random(&mut rng, &preset);
        let row_bits = BitVec::from_bytes(
            &rng.gen_range(i64::MIN, i64::MAX).to_be_bytes()
        );
        let col_bits = BitVec::from_bytes(
            &rng.gen_range(i64::MIN, i64::MAX).to_be_bytes()
        );

        let mut col = 0;

        for (display_row, display_col) in row_bits.iter().zip(col_bits.iter()) {
            if col < (*COLS - 1) && display_row {
                svg.push_str(format!(
                    "<path d=\"M {} {} L {} {}\" stroke=\"{}\" stroke-width=\"2\" />",
                    col * GAP + GAP,
                    row * GAP,
                    col * GAP + GAP,
                    row * GAP + GAP,
                    col_colors.get(col).unwrap().hsl(),
                ).as_str());
            }

            if row < (*ROWS - 1) && display_col {
                svg.push_str(format!(
                    "<path d=\"M {} {} L {} {}\" stroke=\"{}\" stroke-width=\"2\" />",
                    col * GAP,
                    row * GAP + GAP,
                    col * GAP + GAP,
                    row * GAP + GAP,
                    row_color.hsl(),
                ).as_str());
            }

            col = col + 1;
        }
    }

    svg.push_str(format!(
        "<text x=\"{}\" y=\"{}\"
            dominant-baseline=\"hanging\"
            text-anchor=\"end\"
            font-family=\"sans-serif\"
            font-size=\"{}\"
            font-weight=\"lighter\"
            fill=\"lightgrey\"
        >{}:{}</text>",
        WIDTH,
        HEIGHT,
        TEXT_SIZE,
        preset.name(),
        &*COMMIT,
    ).as_str());

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
