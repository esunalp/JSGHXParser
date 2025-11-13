//! Contains functions for sampling and resampling curves and polylines.

/// Resamples two polylines to have the same number of points.
///
/// This function takes two polylines (represented as slices of 3D points) and
/// resamples the shorter polyline to match the point count of the longer one.
/// This is useful for creating surfaces between curves, like in a ruled surface.
///
/// # Arguments
///
/// * `polyline_a` - A slice of 3D points representing the first polyline.
/// * `polyline_b` - A slice of 3D points representing the second polyline.
///
/// # Returns
///
/// A tuple containing two new `Vec<[f32; 3]>`s, representing the resampled
/// polylines with an equal number of points.
pub fn resample_polylines(
    polyline_a: &[[f32; 3]],
    polyline_b: &[[f32; 3]],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    if polyline_a.is_empty() || polyline_b.is_empty() {
        return (polyline_a.to_vec(), polyline_b.to_vec());
    }

    let (longer, shorter) = if polyline_a.len() >= polyline_b.len() {
        (polyline_a, polyline_b)
    } else {
        (polyline_b, polyline_a)
    };

    if shorter.len() < 2 {
        // If the shorter polyline has fewer than 2 points, we can't interpolate.
        // In this case, we just repeat the single point to match the longer list's length.
        let mut resampled_shorter = Vec::with_capacity(longer.len());
        for _ in 0..longer.len() {
            resampled_shorter.push(shorter[0]);
        }
        if polyline_a.len() >= polyline_b.len() {
            return (longer.to_vec(), resampled_shorter);
        } else {
            return (resampled_shorter, longer.to_vec());
        }
    }

    let mut resampled_shorter = Vec::with_capacity(longer.len());
    let shorter_len = (shorter.len() - 1) as f32;
    let longer_len = (longer.len() - 1) as f32;

    for i in 0..longer.len() {
        let t = i as f32 / longer_len; // Normalized position [0, 1] along the longer polyline
        let shorter_t = t * shorter_len; // Corresponding position along the shorter polyline
        let index = shorter_t.floor() as usize;
        let frac = shorter_t.fract();

        if index >= shorter.len() - 1 {
            resampled_shorter.push(shorter[shorter.len() - 1]);
        } else {
            let p0 = shorter[index];
            let p1 = shorter[index + 1];
            let new_point = [
                p0[0] + frac * (p1[0] - p0[0]),
                p0[1] + frac * (p1[1] - p0[1]),
                p0[2] + frac * (p1[2] - p0[2]),
            ];
            resampled_shorter.push(new_point);
        }
    }

    if polyline_a.len() >= polyline_b.len() {
        (longer.to_vec(), resampled_shorter)
    } else {
        (resampled_shorter, longer.to_vec())
    }
}
