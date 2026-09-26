use adamantium_stdlib::math::{PI, PI_F64, cos, sin, tan};

#[test]
fn public_math_api_exposes_trigonometry_and_long_pi() {
    assert_eq!(PI.len(), 3_255);
    assert!((sin(PI_F64 / 2.0) - 1.0).abs() < 1e-12);
    assert!((cos(0.0) - 1.0).abs() < 1e-12);
    assert!(tan(0.0).abs() < 1e-12);
}
