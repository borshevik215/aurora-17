pub fn calibration_matches(
    ra: f32,
    dec: f32,
) -> bool {
    (ra - 0.084).abs()
        < 0.0005
        &&
    (dec + 0.031).abs()
        < 0.0005
}
