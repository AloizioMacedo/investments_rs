use anyhow::Result;
use geo::{ConvexHull, MultiPoint, Point};
use indicatif::ProgressBar;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use plotly::{common::Mode, Layout, Plot, Scatter};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

use crate::{
    config::get_config,
    timeseries::{AllTimeSeries, Portfolio, TimeSeries},
};

struct Statistics {
    splits: Vec<Vec<f64>>,
    volatilities: Vec<f64>,
    average_returns: Vec<f64>,
    returns_at_end: Vec<f64>,
    sharpe_ratios: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
struct Allocation {
    allocations: HashMap<String, f64>,
    sharpe_ratio: f64,
    expected_returns_at_end: f64,
    average: f64,
    volatility: f64,
}

fn load_timeseries() -> Vec<TimeSeries> {
    let path = Path::new("data/03_models/models.json");
    let timeseries = std::fs::read_to_string(path).unwrap();

    let all_ts: AllTimeSeries = serde_json::from_str(&timeseries).unwrap();

    all_ts.timeseries
}

fn load_cdi() -> TimeSeries {
    let path = Path::new("data/03_models/cdi.json");
    let timeseries = std::fs::read_to_string(path).unwrap();

    serde_json::from_str(&timeseries).unwrap()
}

fn get_possible_splits() -> impl Iterator<Item = Vec<f64>> {
    let config = get_config();

    let min_gran = config.portfolio.split_granularity;
    let n_funds = config.portfolio.number_of_funds;

    let total = (1.0 / min_gran) as usize;

    let granularity = (0..total).map(|x| x as f64 * min_gran).collect_vec();

    std::iter::repeat(granularity)
        .take(n_funds - 1)
        .multi_cartesian_product()
        .filter_map(|mut x| {
            let s = x.iter().sum::<f64>();

            if s <= 1.0 {
                x.push(1.0 - s);

                Some(x)
            } else {
                None
            }
        })
}

fn get_statistics_from_splits(
    risk_free: &TimeSeries,
    funds: &[TimeSeries],
    possible_splits: impl Iterator<Item = Vec<f64>>,
) -> Statistics {
    let mut splits = Vec::new();
    let mut volatilities = Vec::new();
    let mut average_returns = Vec::new();
    let mut returns_at_end = Vec::new();
    let mut sharpe_ratios = Vec::new();

    let possible_splits = possible_splits.collect_vec();

    let pb = ProgressBar::new(possible_splits.len() as u64);
    for possible_split in possible_splits {
        let p = Portfolio::new(funds.to_vec(), possible_split.clone()).unwrap();

        volatilities.push(p.std());
        average_returns.push(p.average());
        returns_at_end.push(p.calculate_value_at_end(1.0));
        sharpe_ratios.push(p.sharpe_ratio(risk_free));
        splits.push(possible_split);
        pb.inc(1);
    }
    pb.finish();

    Statistics {
        splits,
        volatilities,
        average_returns,
        returns_at_end,
        sharpe_ratios,
    }
}

fn build_splits_hashmap<'a>(
    xs: &'a [f64],
    ys: &'a [f64],
    splits: &'a [Vec<f64>],
) -> HashMap<(OrderedFloat<f64>, OrderedFloat<f64>), &'a Vec<f64>> {
    let mut hashmap = HashMap::new();

    for i in 0..xs.len() {
        hashmap.insert((OrderedFloat(xs[i]), OrderedFloat(ys[i])), &splits[i]);
    }

    hashmap
}

fn recover_splits<'a>(
    hm: &'a HashMap<(OrderedFloat<f64>, OrderedFloat<f64>), &'a Vec<f64>>,
    xs: &'a [f64],
    ys: &'a [f64],
) -> Vec<&'a Vec<f64>> {
    let mut splits = Vec::new();

    for (x, y) in xs.iter().zip(ys) {
        splits.push(hm[&(OrderedFloat(*x), OrderedFloat(*y))])
    }

    splits
}

fn get_best_funds() -> Vec<TimeSeries> {
    let mut funds = load_timeseries();

    funds.sort_by(|t1, t2| {
        t1.average_returns()
            .partial_cmp(&t2.average_returns())
            .unwrap()
    });
    funds.reverse();

    let config = get_config();
    let n = config.portfolio.number_of_funds;

    funds[0..n].to_vec()
}

pub fn main() -> Result<()> {
    let funds = get_best_funds();
    let cdi = load_cdi();
    let possible_splits = get_possible_splits();

    let statistics = get_statistics_from_splits(&cdi, &funds, possible_splits);
    let splits_as_text = statistics
        .splits
        .iter()
        .map(|x| format!("Split: {:.2?}", x))
        .collect::<Vec<_>>();

    // Efficient Frontier

    let scatter = Scatter::new(
        statistics.volatilities.clone(),
        statistics.average_returns.clone(),
    )
    .mode(Mode::Markers)
    .hover_text_array(splits_as_text.clone());

    let mut plot = Plot::new();

    plot.add_trace(scatter);
    let layout = Layout::new().title("<b>Efficient Frontier</b>".into());
    plot.set_layout(layout);

    let splits_hm = build_splits_hashmap(
        &statistics.volatilities,
        &statistics.average_returns,
        &statistics.splits,
    );

    let html = plot.to_html();

    let path = Path::new("data/04_outputs/efficient_frontier.html");
    std::fs::write(path, html)?;

    // let path = Path::new("data/04_outputs/efficient_frontier.png");
    // plot.write_image(path, plotly::ImageFormat::PNG, 1920, 1080, 1.0);

    // Convex Hull
    let points = statistics
        .volatilities
        .iter()
        .zip(&statistics.average_returns)
        .map(|(x, y)| Point::new(*x, *y))
        .collect();
    let x = MultiPoint::new(points);
    let ch = x.convex_hull();
    let (x, y): (Vec<f64>, Vec<f64>) = ch.exterior().points().map(|p| p.x_y()).unzip();

    let splits_for_ch = recover_splits(&splits_hm, &x, &y);
    let splits_as_text_for_ch = splits_for_ch
        .iter()
        .map(|x| format!("Splits: {:.2?}", x))
        .collect();

    let scatter = Scatter::new(x, y)
        .mode(Mode::Markers)
        .hover_text_array(splits_as_text_for_ch);

    let mut plot = Plot::new();
    plot.add_trace(scatter);
    let layout = Layout::new().title("<b>Convex hull</b>".into());
    plot.set_layout(layout);

    let html = plot.to_html();

    let path = Path::new("data/04_outputs/convex_hull.html");
    std::fs::write(path, html)?;

    // let path = Path::new("data/04_outputs/convex_hull.png");
    // plot.write_image(path, plotly::ImageFormat::PNG, 1920, 1080, 1.0);

    // Returns
    let scatter = Scatter::new(
        statistics.volatilities.clone(),
        statistics.returns_at_end.clone(),
    )
    .mode(Mode::Markers)
    .hover_text_array(splits_as_text);

    let mut plot = Plot::new();

    plot.add_trace(scatter);
    let layout = Layout::new().title("<b>Risk / Return</b>".into());
    plot.set_layout(layout);

    let html = plot.to_html();

    let path = Path::new("data/04_outputs/risk_return.html");
    std::fs::write(path, html)?;

    // let path = Path::new("data/04_outputs/risk_return.png");
    // plot.write_image(path, plotly::ImageFormat::PNG, 1920, 1080, 1.0);

    let idx = statistics
        .sharpe_ratios
        .iter()
        .enumerate()
        .max_by(|(_, x), (_, y)| x.partial_cmp(y).unwrap())
        .unwrap()
        .0;

    let best_split = &statistics.splits[idx];
    let allocations = HashMap::from_iter(
        funds
            .iter()
            .map(|f| f.id.to_string())
            .zip(best_split.iter().copied()),
    );

    let allocation = Allocation {
        allocations,
        average: statistics.average_returns[idx],
        expected_returns_at_end: statistics.returns_at_end[idx],
        sharpe_ratio: statistics.sharpe_ratios[idx],
        volatility: statistics.volatilities[idx],
    };

    let jsonified_allocation = serde_json::to_string(&allocation)?;
    let path = Path::new("data/05_reporting/allocation.json");

    std::fs::write(path, jsonified_allocation)?;

    Ok(())
}
