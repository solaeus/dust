#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_power(base: i64, power: i64) -> i64 {
    base.pow(power as u32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn float_power(base: f64, power: f64) -> f64 {
    base.powf(power)
}
