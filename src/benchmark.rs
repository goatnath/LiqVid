//! Reference values and comparison helpers for the Re=100 lid-driven cavity benchmark.
//! Reference: Ghia, Ghia & Shin, Journal of Computational Physics 48 (1982), 387–411.

/// Horizontal velocity u/U along the vertical centerline x/L = 0.5, for Re = 100.
pub const GHIA_RE100_U: &[(f64, f64)] = &[
    (1.0000, 1.00000),
    (0.9766, 0.84123),
    (0.9688, 0.78871),
    (0.9609, 0.73722),
    (0.9531, 0.68717),
    (0.8516, 0.23151),
    (0.7344, 0.00332),
    (0.6172, -0.13641),
    (0.5000, -0.20581),
    (0.4531, -0.21090),
    (0.2813, -0.15662),
    (0.1719, -0.10150),
    (0.1016, -0.06434),
    (0.0703, -0.04775),
    (0.0625, -0.04192),
    (0.0547, -0.03717),
    (0.0000, 0.00000),
];

/// Linear interpolation of an ascending (coordinate, value) profile.
/// Rejects malformed profiles and extrapolation outside the supplied interval.
pub fn interpolate_profile(points: &[(f64, f64)], coordinate: f64) -> Option<f64> {
    if points.len() < 2
        || !coordinate.is_finite()
        || points.iter().any(|(x, y)| !x.is_finite() || !y.is_finite())
        || points.windows(2).any(|w| w[0].0 >= w[1].0)
        || coordinate < points[0].0
        || coordinate > points.last()?.0
    {
        return None;
    }
    let pair = points
        .windows(2)
        .find(|w| coordinate >= w[0].0 && coordinate <= w[1].0)?;
    let t = (coordinate - pair[0].0) / (pair[1].0 - pair[0].0);
    Some(pair[0].1 + t * (pair[1].1 - pair[0].1))
}

/// One comparison at a published Ghia sample coordinate.
#[derive(serde::Serialize)]
pub struct CenterlineComparison {
    pub y: f64,
    pub reference: f64,
    pub simulated: f64,
    pub absolute_error: f64,
}

/// Interpolate the computed profile at all 17 published coordinates.
/// This avoids comparing a coarse interpolation of the reference to solver nodes.
pub fn compare_centerline(samples: &[(f64, f64)]) -> Option<Vec<CenterlineComparison>> {
    GHIA_RE100_U
        .iter()
        .map(|&(y, reference)| {
            let simulated = interpolate_profile(samples, y)?;
            Some(CenterlineComparison {
                y,
                reference,
                simulated,
                absolute_error: (simulated - reference).abs(),
            })
        })
        .collect()
}

/// Maximum absolute u/U error at the published Ghia coordinates.
pub fn max_centerline_error(samples: &[(f64, f64)]) -> Option<f64> {
    Some(
        compare_centerline(samples)?
            .iter()
            .map(|p| p.absolute_error)
            .fold(0.0, f64::max),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_table_contains_canonical_re100_centerline_values() {
        assert_eq!(GHIA_RE100_U.len(), 17);
        assert!((GHIA_RE100_U[8].1 - (-0.20581)).abs() < 1e-8);
        let mut profile = GHIA_RE100_U.to_vec();
        profile.reverse();
        assert!(max_centerline_error(&profile).unwrap() < 1e-14);
    }

    #[test]
    fn comparison_interpolates_simulation_at_reference_coordinates() {
        let profile = [(0.0, 0.0), (0.4, 0.4), (1.0, 1.0)];
        let comparisons = compare_centerline(&profile).unwrap();
        assert_eq!(comparisons.len(), 17);
        let middle = &comparisons[8];
        assert!((middle.simulated - 0.5).abs() < 1e-12);
        assert!((middle.absolute_error - 0.70581).abs() < 1e-12);
        assert!(max_centerline_error(&profile).unwrap() >= middle.absolute_error);
    }

    #[test]
    fn interpolation_rejects_missing_coverage_and_nonfinite_or_unordered_data() {
        for profile in [
            vec![],
            vec![(0.5, 0.0)],
            vec![(0.2, 0.0), (0.8, 1.0)],
            vec![(1.0, 1.0), (0.0, 0.0)],
            vec![(0.0, 0.0), (0.0, 1.0)],
            vec![(0.0, f64::NAN), (1.0, 1.0)],
        ] {
            assert!(compare_centerline(&profile).is_none());
        }
        assert_eq!(
            interpolate_profile(&[(0.0, 0.0), (1.0, 1.0)], 0.0),
            Some(0.0)
        );
        assert_eq!(
            interpolate_profile(&[(0.0, 0.0), (1.0, 1.0)], 1.0),
            Some(1.0)
        );
        assert!(interpolate_profile(&[(0.0, 0.0), (1.0, 1.0)], f64::NAN).is_none());
    }
}
