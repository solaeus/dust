#[unsafe(no_mangle)]
pub unsafe extern "C" fn byte_power(base: u8, power: u8) -> i64 {
    base.saturating_pow(power as u32) as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_power(base: i64, power: i64) -> i64 {
    base.saturating_pow(power as u32)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn float_power(base: f64, power: f64) -> i64 {
    base.powf(power).to_bits() as i64
}
