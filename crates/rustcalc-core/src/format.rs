//! Display formatting for numbers.

/// Values with magnitude at or above this use scientific notation.
const SCI_UPPER: f64 = 1e12;
/// Non-zero values with magnitude below this use scientific notation.
const SCI_LOWER: f64 = 1e-9;

/// Format a value as a raw (ungrouped) display string.
///
/// Finite values round to 12 decimal places, or switch to scientific
/// notation outside `[1e-9, 1e12)`. Non-finite inputs are shown as
/// `"NaN"`, `"∞"` or `"-∞"`.
pub fn format_number(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "∞"
        } else {
            "-∞"
        }
        .to_string();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    let magnitude = value.abs();
    if magnitude >= SCI_UPPER || magnitude < SCI_LOWER {
        scientific(value)
    } else {
        trim_trailing_zeros(format!("{value:.12}"))
    }
}

fn scientific(value: f64) -> String {
    let raw = format!("{value:.10e}");
    let mut mantissa = String::with_capacity(raw.len());
    let mut exponent = String::new();
    let mut seen_e = false;
    for ch in raw.chars() {
        if seen_e {
            exponent.push(ch);
        } else if ch == 'e' {
            seen_e = true;
        } else {
            mantissa.push(ch);
        }
    }
    format!("{}e{exponent}", trim_trailing_zeros(mantissa))
}

fn trim_trailing_zeros(mut text: String) -> String {
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

/// Insert thousands separators into the integer part of `text`.
pub(crate) fn group_digits(text: &str) -> String {
    if text.contains('e') {
        return text.to_string();
    }
    let (sign, unsigned) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text),
    };
    let (integer, fraction) = match unsigned.split_once('.') {
        Some((int_part, frac_part)) => (int_part, Some(frac_part)),
        None => (unsigned, None),
    };
    let mut grouped = String::with_capacity(text.len() + text.len() / 3);
    for (index, ch) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.insert_str(0, sign);
    if let Some(frac_part) = fraction {
        grouped.push('.');
        grouped.push_str(frac_part);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_formats_as_single_zero() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(-0.0), "0");
    }

    #[test]
    fn fixed_notation_trims_trailing_zeros() {
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(1.0 / 3.0), "0.333333333333");
        assert_eq!(format_number(999_999_999_999.5), "999999999999.5");
        assert_eq!(format_number(1_000_000.5), "1000000.5");
    }

    #[test]
    fn scientific_notation_above_upper_bound() {
        assert_eq!(format_number(1e12), "1e12");
        assert_eq!(format_number(1.23e15), "1.23e15");
        assert_eq!(format_number(-2.5e20), "-2.5e20");
    }

    #[test]
    fn scientific_notation_below_lower_bound() {
        assert_eq!(format_number(1e-9), "0.000000001");
        assert_eq!(format_number(9.9e-10), "9.9e-10");
        assert_eq!(format_number(-4.2e-30), "-4.2e-30");
    }

    #[test]
    fn non_finite_values_are_labelled() {
        assert_eq!(format_number(f64::NAN), "NaN");
        assert_eq!(format_number(f64::INFINITY), "∞");
        assert_eq!(format_number(f64::NEG_INFINITY), "-∞");
    }

    #[test]
    fn trimming_keeps_integers_without_decimal_point() {
        assert_eq!(trim_trailing_zeros("1000".to_string()), "1000");
    }

    #[test]
    fn trimming_removes_fraction_and_bare_point() {
        assert_eq!(trim_trailing_zeros("1.500".to_string()), "1.5");
        assert_eq!(trim_trailing_zeros("1.000".to_string()), "1");
    }

    #[test]
    fn grouping_inserts_separators() {
        assert_eq!(group_digits("1234567"), "1,234,567");
        assert_eq!(group_digits("123"), "123");
        assert_eq!(group_digits("100"), "100");
        assert_eq!(group_digits("-1234.5"), "-1,234.5");
        assert_eq!(group_digits("1234.567"), "1,234.567");
        assert_eq!(group_digits("0.5"), "0.5");
        assert_eq!(group_digits("12"), "12");
        assert_eq!(group_digits(""), "");
    }

    #[test]
    fn grouping_skips_scientific_notation() {
        assert_eq!(group_digits("1e12"), "1e12");
        assert_eq!(group_digits("-1.23e20"), "-1.23e20");
    }
}
