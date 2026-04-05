use std::num::NonZeroU8;

use lexical_parse_float::{
    self, FromLexicalWithOptions as _, NumberFormatBuilder as FloatFormatBuilder,
    Options as FloatOptions,
};
use lexical_parse_integer::{
    self, FromLexicalWithOptions as _, NumberFormatBuilder as IntFormatBuilder,
    Options as IntOptions,
};

use crate::compiler::error::CompileError;

const INTEGER: u128 = IntFormatBuilder::new()
    .digit_separator(NonZeroU8::new(b'_'))
    .required_digits(true)
    .no_positive_mantissa_sign(true)
    .internal_digit_separator(true)
    .trailing_digit_separator(true)
    .consecutive_digit_separator(true)
    .build_strict();

const HEX: u128 = IntFormatBuilder::new()
    .mantissa_radix(16)
    .digit_separator(NonZeroU8::new(b'_'))
    .required_digits(true)
    .no_positive_mantissa_sign(true)
    .internal_digit_separator(true)
    .trailing_digit_separator(true)
    .consecutive_digit_separator(true)
    .build_strict();

const FLOAT: u128 = FloatFormatBuilder::new()
    .digit_separator(NonZeroU8::new(b'_'))
    .required_digits(true)
    .no_positive_mantissa_sign(true)
    .no_special(true)
    .internal_digit_separator(true)
    .trailing_digit_separator(true)
    .consecutive_digit_separator(true)
    .build_strict();

const SMALL: IntOptions = IntOptions::builder().no_multi_digit(true).build_strict();
const LARGE: IntOptions = IntOptions::builder().no_multi_digit(false).build_strict();

pub fn create_i8_from_decimal(text: &str) -> Result<i8, CompileError> {
    i8::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_i16_from_decimal(text: &str) -> Result<i16, CompileError> {
    i16::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_i32_from_decimal(text: &str) -> Result<i32, CompileError> {
    i32::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_i64_from_decimal(text: &str) -> Result<i64, CompileError> {
    i64::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_i128_from_decimal(text: &str) -> Result<i128, CompileError> {
    i128::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_isize_from_decimal(text: &str) -> Result<isize, CompileError> {
    isize::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_u8_from_decimal(text: &str) -> Result<u8, CompileError> {
    u8::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_u16_from_decimal(text: &str) -> Result<u16, CompileError> {
    u16::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_u32_from_decimal(text: &str) -> Result<u32, CompileError> {
    u32::from_lexical_with_options::<INTEGER>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}

pub fn create_u64_from_decimal(text: &str) -> Result<u64, CompileError> {
    u64::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_u128_from_decimal(text: &str) -> Result<u128, CompileError> {
    u128::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_usize_from_decimal(text: &str) -> Result<usize, CompileError> {
    usize::from_lexical_with_options::<INTEGER>(text.as_bytes(), &LARGE)
        .map_err(CompileError::ValueCreation)
}

pub fn create_f32_from_decimal(text: &str) -> Result<f32, CompileError> {
    f32::from_lexical_with_options::<FLOAT>(text.as_bytes(), &FloatOptions::new())
        .map_err(CompileError::ValueCreation)
}

pub fn create_f64_from_decimal(text: &str) -> Result<f64, CompileError> {
    f64::from_lexical_with_options::<FLOAT>(text.as_bytes(), &FloatOptions::new())
        .map_err(CompileError::ValueCreation)
}

pub fn create_u8_from_hexadecimal(text: &str) -> Result<u8, CompileError> {
    u8::from_lexical_with_options::<HEX>(text.as_bytes(), &SMALL)
        .map_err(CompileError::ValueCreation)
}
