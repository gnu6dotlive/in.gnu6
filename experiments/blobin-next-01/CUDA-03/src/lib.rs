//! CUDA-03 resident geometry and timing proofs; no F2 authority mutation.
pub mod cuda;
pub mod vulkan;

pub fn summary(series: &[f64]) -> Result<(f64, f64, f64), String> {
    if series.is_empty() || series.iter().any(|x| !x.is_finite() || *x < 0.0) {
        return Err("CUDA03 INVALID BENCHMARK SAMPLES".into());
    }
    let mut values = series.to_vec();
    values.sort_by(f64::total_cmp);
    let q = |fraction: f64| {
        let i = ((values.len() as f64 * fraction).ceil() as usize)
            .saturating_sub(1)
            .min(values.len() - 1);
        values[i]
    };
    Ok((
        values.iter().sum::<f64>() / values.len() as f64,
        q(0.5),
        q(0.95),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
    use blobin_f2_topology::Scene;
    #[test]
    fn measured_quantiles() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(summary(&a).unwrap(), (5.5, 5.0, 10.0));
    }
    #[test]
    fn invalid_samples_fail_closed() {
        assert!(summary(&[]).is_err());
        assert!(summary(&[1.0, f64::NAN]).is_err());
    }
    #[test]
    fn geometry_update_requires_recalculation() {
        let scene = Scene::default();
        let old = Request::from_f2(scene, generate_scene(16)).unwrap();
        let mut shapes = generate_scene(16);
        shapes[0].radius += 0.080;
        let new = Request::from_f2(scene, shapes).unwrap();
        assert!(compare(&cpu(&old).unwrap(), &cpu(&new).unwrap()).is_err());
        assert_eq!(old.authority(), new.authority());
    }
    #[test]
    fn missing_cuda_library_fails_closed() {
        let r = Request::from_f2(Scene::default(), generate_scene(16)).unwrap();
        assert!(cuda::Resident::new(&r, std::path::Path::new("/missing/blobin03.so")).is_err());
    }
    #[test]
    fn input_limits_inherited() {
        assert!(Request::from_f2(Scene::default(), generate_scene(257)).is_err());
    }
}
