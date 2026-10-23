//! Reference values and comparison helpers for the Re=100 lid-driven cavity benchmark.
//! Reference: Ghia, Ghia & Shin, Journal of Computational Physics 48 (1982), 387–411.

/// Horizontal velocity u/U along the vertical centerline x/L = 0.5, for Re = 100.
pub const GHIA_RE100_U: &[(f64, f64)] = &[
    (1.0000, 1.00000), (0.9766, 0.84123), (0.9688, 0.78871),
    (0.9609, 0.73722), (0.9531, 0.68717), (0.8516, 0.23151),
    (0.7344, 0.00332), (0.6172, -0.13641), (0.5000, -0.20581),
    (0.4531, -0.21090), (0.2813, -0.15662), (0.1719, -0.10150),
    (0.1016, -0.06434), (0.0703, -0.04775), (0.0625, -0.04192),
    (0.0547, -0.03717), (0.0000, 0.00000),
];

/// Compare sampled, normalized centerline velocities against Ghia et al.
/// Samples are (y/L, u/U); interpolation is linear between reference points.
pub fn max_centerline_error(samples: &[(f64, f64)]) -> Option<f64> {
    if samples.is_empty() { return None; }
    samples.iter().map(|&(y, u)| {
        let mut points = GHIA_RE100_U.to_vec();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        let reference = if y <= points[0].0 { points[0].1 }
        else if y >= points[points.len()-1].0 { points[points.len()-1].1 }
        else {
            let pair = points.windows(2).find(|w| y >= w[0].0 && y <= w[1].0)?;
            let t = (y - pair[0].0) / (pair[1].0 - pair[0].0);
            pair[0].1 + t * (pair[1].1 - pair[0].1)
        };
        Some((u - reference).abs())
    }).collect::<Option<Vec<_>>>().map(|errors| errors.into_iter().fold(0.0, f64::max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_table_contains_canonical_re100_centerline_values() {
        assert_eq!(GHIA_RE100_U.len(), 17);
        assert!((GHIA_RE100_U[8].1 - (-0.20581)).abs() < 1e-8);
        assert_eq!(max_centerline_error(&[(0.5, -0.20581)]), Some(0.0));
    }

    #[test]
    fn comparison_reports_error_for_a_mismatched_profile() {
        assert!(max_centerline_error(&[(0.5, 0.0)]).unwrap() > 0.2);
    }
}
