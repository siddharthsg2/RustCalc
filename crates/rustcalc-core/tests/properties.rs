use proptest::prelude::*;
use rustcalc_core::{Calculator, Op, format_number};

fn type_int(c: &mut Calculator, value: u32) {
    for ch in value.to_string().chars() {
        c.input_digit(ch as u8 - b'0');
    }
}

fn ungrouped(display: String) -> String {
    display.replace(',', "")
}

proptest! {
    #[test]
    fn formatted_numbers_round_trip(value in -1e15f64..1e15f64) {
        let text = format_number(value);
        let parsed: f64 = text.parse().expect("formatted numbers must parse back");
        let tolerance = value.abs().max(1.0) * 1e-9;
        prop_assert!((parsed - value).abs() <= tolerance);
    }

    #[test]
    fn addition_is_commutative(a in 0u32..1_000_000, b in 0u32..1_000_000) {
        let mut first = Calculator::new();
        type_int(&mut first, a);
        first.set_op(Op::Add);
        type_int(&mut first, b);
        first.equals();

        let mut second = Calculator::new();
        type_int(&mut second, b);
        second.set_op(Op::Add);
        type_int(&mut second, a);
        second.equals();

        prop_assert_eq!(first.display(), second.display());
    }

    #[test]
    fn multiplication_and_division_are_inverse(a in 1u32..100_000, b in 1u32..100_000) {
        let mut c = Calculator::new();
        type_int(&mut c, a);
        c.set_op(Op::Multiply);
        type_int(&mut c, b);
        c.equals();
        c.set_op(Op::Divide);
        type_int(&mut c, b);
        c.equals();
        prop_assert_eq!(ungrouped(c.display()), a.to_string());
    }

    #[test]
    fn subtraction_then_addition_restores(value in 0u32..1_000_000, offset in 0u32..1_000_000) {
        let mut c = Calculator::new();
        type_int(&mut c, value);
        c.set_op(Op::Subtract);
        type_int(&mut c, offset);
        c.equals();
        c.set_op(Op::Add);
        type_int(&mut c, offset);
        c.equals();
        prop_assert_eq!(ungrouped(c.display()), value.to_string());
    }
}
