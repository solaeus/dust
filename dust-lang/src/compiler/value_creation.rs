use crate::{compiler::error::CompileError, syntax::reader::SyntaxReader};

macro_rules! create_integer_from_decimal_bytes {
    ($bytes:expr, $type:ty, $reader:expr) => {{
        let mut value: $type = 0;
        let mut negate = false;

        let digits = if $bytes.first() == Some(&b'-') {
            negate = true;
            &$bytes[1..]
        } else {
            $bytes
        };

        for &byte in digits {
            if byte == b'_' {
                continue;
            }

            let digit = byte - b'0';

            value = value
                .checked_mul(10)
                .and_then(|value| value.checked_add(digit as $type))
                .ok_or_else(|| CompileError::ConstantValueOverflow {
                    position: $reader.position(),
                })?;
        }

        if negate {
            value = value.checked_neg().ok_or_else(|| CompileError::ConstantValueOverflow {
                position: $reader.position(),
            })?;
        }

        Ok(value)
    }};
}

macro_rules! create_unsigned_integer_from_hexadecimal {
    ($bytes:expr, $type:ty, $reader:expr) => {{
        let mut value: $type = 0;

        for &byte in $bytes {
            if byte == b'_' {
                continue;
            }

            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => continue,
            };

            value = value
                .checked_mul(16)
                .and_then(|value| value.checked_add(digit))
                .ok_or_else(|| CompileError::ConstantValueOverflow {
                    position: $reader.position(),
                })?;
        }

        Ok(value)
    }};
}

macro_rules! create_float_from_decimal {
    ($bytes:expr, $type:ty, $reader:expr) => {{
        if $bytes == b"NaN" {
            return Ok(<$type>::NAN);
        }

        if $bytes == b"Infinity" {
            return Ok(<$type>::INFINITY);
        }

        if $bytes == b"-Infinity" {
            return Ok(<$type>::NEG_INFINITY);
        }

        let mut negate = false;

        let digits = if $bytes.first() == Some(&b'-') {
            negate = true;
            &$bytes[1..]
        } else {
            $bytes
        };

        let mut mantissa = 0_u64;

        let mut fraction_digits = 0_i32;
        let mut in_fraction = false;

        let mut exponent = 0_i32;
        let mut exponent_value = 0_i32;
        let mut in_exponent = false;
        let mut in_negative_exponent = false;

        for &byte in digits {
            if byte == b'_' {
                continue;
            }

            if in_exponent {
                if byte == b'-' {
                    in_negative_exponent = true;
                } else if byte == b'+' {
                } else if byte.is_ascii_digit() {
                    let digit = byte - b'0';
                    exponent_value = exponent_value
                        .checked_mul(10)
                        .and_then(|value| value.checked_add(digit as i32))
                        .ok_or_else(|| CompileError::ConstantValueOverflow {
                            position: $reader.position(),
                        })?;
                }

                continue;
            }

            if byte == b'.' {
                in_fraction = true;

                continue;
            }

            if byte == b'e' || byte == b'E' {
                in_exponent = true;

                continue;
            }

            let digit = (byte - b'0') as u64;

            mantissa = mantissa
                .checked_mul(10)
                .and_then(|value| value.checked_add(digit))
                .ok_or_else(|| CompileError::ConstantValueOverflow {
                    position: $reader.position(),
                })?;

            if in_fraction {
                fraction_digits += 1;
            }
        }

        if in_negative_exponent {
            exponent = -exponent;
        }

        exponent = exponent - fraction_digits + exponent_value;

        let mut value = (mantissa as $type) * (10 as $type).powi(exponent);

        if negate {
            value = -value;
        }

        Ok(value)
    }};
}

pub fn create_i8_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<i8, CompileError> {
    create_integer_from_decimal_bytes!(bytes, i8, reader)
}

pub fn create_i16_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<i16, CompileError> {
    create_integer_from_decimal_bytes!(bytes, i16, reader)
}

pub fn create_i32_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<i32, CompileError> {
    create_integer_from_decimal_bytes!(bytes, i32, reader)
}

pub fn create_i64_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<i64, CompileError> {
    create_integer_from_decimal_bytes!(bytes, i64, reader)
}

pub fn create_i128_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<i128, CompileError> {
    create_integer_from_decimal_bytes!(bytes, i128, reader)
}

pub fn create_u8_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<u8, CompileError> {
    create_integer_from_decimal_bytes!(bytes, u8, reader)
}

pub fn create_u16_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<u16, CompileError> {
    create_integer_from_decimal_bytes!(bytes, u16, reader)
}

pub fn create_u32_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<u32, CompileError> {
    create_integer_from_decimal_bytes!(bytes, u32, reader)
}

pub fn create_u64_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<u64, CompileError> {
    create_integer_from_decimal_bytes!(bytes, u64, reader)
}

pub fn create_u128_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<u128, CompileError> {
    create_integer_from_decimal_bytes!(bytes, u128, reader)
}

pub fn create_usize_from_decimal(
    bytes: &[u8],
    reader: SyntaxReader,
) -> Result<usize, CompileError> {
    create_integer_from_decimal_bytes!(bytes, usize, reader)
}

pub fn create_f32_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<f32, CompileError> {
    create_float_from_decimal!(bytes, f32, reader)
}

pub fn create_f64_from_decimal(bytes: &[u8], reader: SyntaxReader) -> Result<f64, CompileError> {
    create_float_from_decimal!(bytes, f64, reader)
}

pub fn create_u8_from_hexadecimal(bytes: &[u8], reader: SyntaxReader) -> Result<u8, CompileError> {
    create_unsigned_integer_from_hexadecimal!(bytes, u8, reader)
}

pub fn create_char(text: &str) -> Result<char, CompileError> {
    Ok(text.chars().next().unwrap_or_default())
}
