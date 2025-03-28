// https://www.coingecko.com/en/api/documentation

#![warn(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use std::error;
use std::fs::File;
use std::slice::Iter;

use chrono::serde::ts_milliseconds;
use chrono::{DateTime, NaiveDate, Utc};
use clap::{arg, crate_name, crate_version, value_parser, Command};
use plotters::backend::SVGBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::Circle;
use plotters::series::{LineSeries, PointSeries};
use plotters::style::{Color, FontTransform, IntoFont, TextStyle, BLACK, BLUE, RED, WHITE};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    prices: Vec<Datum>,
    market_caps: Vec<Datum>,
    total_volumes: Vec<Datum>,
}

impl Data {
    fn iter_prices(&self) -> Iter<'_, Datum> {
        self.prices.iter()
    }

    fn iter_market_caps(&self) -> Iter<'_, Datum> {
        self.market_caps.iter()
    }

    fn iter_total_volumes(&self) -> Iter<'_, Datum> {
        self.total_volumes.iter()
    }
}

#[derive(Debug, Deserialize)]
struct Datum(
    #[serde(with = "ts_milliseconds")] DateTime<Utc>,
    Option<f64>,
);

impl Datum {
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.0
    }

    fn price(&self) -> Option<f64> {
        self.1
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .args(&[
            arg!(--fetch "fetch from API"),
            arg!(--days <DAYS> "past days")
                .value_parser(value_parser!(u32))
                .default_value("365"),
        ])
        .get_matches();

    let data: Data = if matches.contains_id("fetch") {
        let resp = ureq::get("https://api.coingecko.com/api/v3/coins/ethereum/market_chart")
            .header("accept", "application/json")
            .query("vs_currency", "usd")
            .query(
                "days",
                matches
                    .get_one::<u32>("days")
                    .expect("cannot fail because of default value")
                    .to_string(),
            )
            .call()?;

        // in case of an error HTTP status code, the body is not
        // accessible in ureq 3, see
        // https://github.com/algesten/ureq/issues/997

        serde_json::from_reader(resp.into_body().into_reader())?
    } else {
        let file = File::open("/home/stephan/Downloads/response_1668851750741.json")?;
        serde_json::from_reader(file)?
    };

    let root = SVGBackend::new("graph.svg", (1024, 768)).into_drawing_area();

    let sub_roots = root.split_evenly((2, 1));

    let x_min = *data.iter_prices().map(Datum::timestamp).min().unwrap();
    let x_max = *data.iter_prices().map(Datum::timestamp).max().unwrap();
    let y_min = data
        .iter_prices()
        .filter_map(Datum::price)
        .min_by(f64::total_cmp)
        .unwrap();
    let y_max = data
        .iter_prices()
        .filter_map(Datum::price)
        .max_by(f64::total_cmp)
        .unwrap();

    let mut chart = ChartBuilder::on(&sub_roots[0])
        .caption("ETH price", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_label_style(
            TextStyle::from(("sans-serif", 10).into_font()).transform(FontTransform::Rotate270),
        )
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter_prices()
                .filter_map(|x| x.price().map(|price| (*x.timestamp(), price))),
            RED,
        ))?
        .label("ETH price in USD");

    let exact_merge_date = DateTime::<Utc>::from_naive_utc_and_offset(
        NaiveDate::from_ymd_opt(2022, 9, 15)
            .expect("valid date")
            .and_hms_opt(6, 43, 0)
            .expect("valid time"),
        Utc,
    );
    let point_data = vec![(exact_merge_date, 1450f64)];

    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            point_data.iter().map(|x| (x.0, x.1)),
            5,
            BLUE,
        ))?
        .label("merge date");

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    let x_min = *data.iter_market_caps().map(Datum::timestamp).min().unwrap();
    let x_max = *data.iter_market_caps().map(Datum::timestamp).max().unwrap();
    let y_min = data
        .iter_market_caps()
        .filter_map(Datum::price)
        .min_by(f64::total_cmp)
        .unwrap();
    let y_max = data
        .iter_market_caps()
        .filter_map(Datum::price)
        .max_by(f64::total_cmp)
        .unwrap();

    let mut chart = ChartBuilder::on(&sub_roots[1])
        .caption("ETH market cap", ("sans-serif", 50).into_font())
        .margin(10)
        .margin_left(55)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_label_style(
            TextStyle::from(("sans-serif", 10).into_font()).transform(FontTransform::Rotate270),
        )
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter_market_caps()
                .filter_map(|x| x.price().map(|price| (*x.timestamp(), price))),
            RED,
        ))?
        .label("ETH market cap in USD");

    let point_data = vec![(exact_merge_date, 1450f64)];

    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            point_data.into_iter(),
            5,
            BLUE,
        ))?
        .label("merge date");

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}
