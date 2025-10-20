#[unsafe(no_mangle)]
pub unsafe extern "C" fn integer_power(base: i64, power: i64) -> i64 {
    base.pow(power as u32)
}
