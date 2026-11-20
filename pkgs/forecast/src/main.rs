// forecast: Commit-volume trend and projection (commit platform, pkgs/forecast)
//! Forecasts commit volume: fits a linear trend to daily commit counts with
//! gradient descent (candle) and projects the next days. Used by the weekly
//! report to flag repositories whose activity is dropping.
//!
//!   forecast 12 15 11 18 20 22 25   # last N days, oldest first

use anyhow::{Result, bail};
use candle_core::{DType, Device, Tensor};
use candle_nn::{Linear, Module, Optimizer, SGD, VarBuilder, VarMap};

/// Fits y = w*x + b on (x, y) pairs and returns (w, b).
fn fit(xs: &[f32], ys: &[f32], steps: usize) -> Result<(f32, f32)> {
    let device = Device::Cpu;
    let x = Tensor::from_slice(xs, (xs.len(), 1), &device)?;
    let y = Tensor::from_slice(ys, (ys.len(), 1), &device)?;

    let vars = VarMap::new();
    let model: Linear =
        candle_nn::linear(1, 1, VarBuilder::from_varmap(&vars, DType::F32, &device))?;
    let mut sgd = SGD::new(vars.all_vars(), 0.05)?;
    for _ in 0..steps {
        let loss = model.forward(&x)?.sub(&y)?.sqr()?.mean_all()?;
        sgd.backward_step(&loss)?;
    }
    let w = model.weight().flatten_all()?.to_vec1::<f32>()?[0];
    let b = model.bias().expect("linear has bias").to_vec1::<f32>()?[0];
    Ok((w, b))
}

/// Projects `days` future daily counts (never negative) from the history.
/// Days are scaled to 0..1 so plain SGD converges quickly.
fn project(history: &[f32], days: usize) -> Result<Vec<f32>> {
    if history.len() < 2 {
        bail!("need at least two days of history");
    }
    let n = history.len() as f32;
    let xs: Vec<f32> = (0..history.len()).map(|i| i as f32 / n).collect();
    let scale = history.iter().cloned().fold(1.0_f32, f32::max);
    let ys: Vec<f32> = history.iter().map(|y| y / scale).collect();
    let (w, b) = fit(&xs, &ys, 3000)?;
    Ok((0..days)
        .map(|d| ((w * (n + d as f32) / n + b) * scale).max(0.0))
        .collect())
}

fn main() -> Result<()> {
    let history: Vec<f32> = std::env::args()
        .skip(1)
        .map(|a| a.parse())
        .collect::<Result<_, _>>()?;
    for (day, value) in project(&history, 7)?.iter().enumerate() {
        println!("day +{}: {value:.1} commits", day + 1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_line() {
        let xs: Vec<f32> = (0..20).map(|i| i as f32 / 10.0).collect();
        let ys: Vec<f32> = xs.iter().map(|x| -2.0 * x + 0.5).collect();
        let (w, b) = fit(&xs, &ys, 3000).unwrap();
        assert!((w + 2.0).abs() < 0.05, "w = {w}");
        assert!((b - 0.5).abs() < 0.05, "b = {b}");
    }

    #[test]
    fn projects_a_rising_trend() {
        let history: Vec<f32> = (0..14).map(|d| 10.0 + 2.0 * d as f32).collect();
        let next = project(&history, 3).unwrap();
        for (i, expected) in [38.0, 40.0, 42.0].iter().enumerate() {
            assert!((next[i] - expected).abs() < 1.5, "{next:?}");
        }
    }

    #[test]
    fn needs_history() {
        assert!(project(&[5.0], 3).is_err());
    }
}
