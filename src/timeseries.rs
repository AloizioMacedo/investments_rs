use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use statrs::statistics::Statistics;

#[derive(Serialize, Deserialize)]
pub struct AllTimeSeries {
    pub timeseries: Vec<TimeSeries>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    pub id: String,
    multipliers: Vec<f64>,
    pub returns: Vec<f64>,
}

impl TimeSeries {
    pub fn new(id: String, returns: Vec<f64>) -> TimeSeries {
        let multipliers = returns.iter().map(|x| 1.0 + x).collect();

        TimeSeries {
            id,
            multipliers,
            returns,
        }
    }

    pub fn subtract(&self, other: &TimeSeries) -> TimeSeries {
        let returns = self
            .returns
            .iter()
            .zip(&other.returns)
            .map(|(x, y)| x - y)
            .collect::<Vec<_>>();

        let id = self.id.clone() + "_" + other.id.as_str();

        let multipliers = returns.iter().map(|x| 1.0 + x).collect();

        TimeSeries {
            id,
            multipliers,
            returns,
        }
    }

    pub fn average_returns(&self) -> f64 {
        self.returns.iter().mean()
    }

    pub fn std_returns(&self) -> f64 {
        self.returns.iter().std_dev()
    }

    pub fn calculate_value_at_end(&self, initial_investment: f64) -> f64 {
        initial_investment * self.multipliers.iter().product::<f64>()
    }
}

pub struct Portfolio {
    _ts: Vec<TimeSeries>,
    _split: Vec<f64>,
    final_ts: TimeSeries,
}

impl Portfolio {
    pub fn new(ts: Vec<TimeSeries>, split: Vec<f64>) -> Result<Portfolio> {
        if ts.len() != split.len() {
            return Err(anyhow!("'ts' and 'split' have different lengths"));
        }

        if !(split.iter().sum::<f64>() == 1.0) {
            return Err(anyhow!("Split does not sum to 1"));
        }

        let returns =
            ts.iter()
                .zip(&split)
                .fold(vec![0.0; ts[0].returns.len()], |mut acc, (ts, split)| {
                    for (i, multiplier) in ts.returns.iter().enumerate() {
                        acc[i] += split * multiplier
                    }
                    acc
                });

        let id = ts
            .iter()
            .map(|x| x.id.clone())
            .collect::<Vec<_>>()
            .join("_");

        let final_ts = TimeSeries::new(id, returns);

        Ok(Portfolio {
            _ts: ts,
            _split: split,
            final_ts,
        })
    }

    pub fn std(&self) -> f64 {
        self.final_ts.std_returns()
    }

    pub fn average(&self) -> f64 {
        self.final_ts.average_returns()
    }

    pub fn calculate_value_at_end(&self, initial_investment: f64) -> f64 {
        self.final_ts.calculate_value_at_end(initial_investment)
    }

    pub fn sharpe_ratio(&self, risk_free: &TimeSeries) -> f64 {
        let excess = self.final_ts.subtract(risk_free);

        excess.average_returns() / excess.std_returns()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_final_value() {
        let ts = TimeSeries::new("".to_string(), vec![0.05, 0.07, 0.03]);

        assert_eq!(
            ts.calculate_value_at_end(1.0),
            1.0 * (1.05) * (1.07) * (1.03)
        );

        assert_eq!(ts.average_returns(), 0.05);
    }
}
