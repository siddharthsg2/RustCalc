use rustcalc_core::{CalcError, Calculator, Op};

fn type_str(c: &mut Calculator, text: &str) {
    for ch in text.chars() {
        match ch {
            '0'..='9' => c.input_digit(ch as u8 - b'0'),
            '.' => c.input_decimal(),
            _ => panic!("unsupported test input: {ch}"),
        }
    }
}

#[test]
fn starts_at_zero_with_no_expression() {
    let c = Calculator::new();
    assert_eq!(c.display(), "0");
    assert_eq!(c.expression(), "");
    assert_eq!(c.error(), None);
    assert!(c.history().is_empty());
}

#[test]
fn typing_digits_builds_an_entry() {
    let mut c = Calculator::new();
    type_str(&mut c, "123");
    assert_eq!(c.display(), "123");
}

#[test]
fn leading_zero_is_replaced() {
    let mut c = Calculator::new();
    type_str(&mut c, "0");
    assert_eq!(c.display(), "0");
    type_str(&mut c, "5");
    assert_eq!(c.display(), "5");
}

#[test]
fn decimal_point_starts_fractional_entry() {
    let mut c = Calculator::new();
    c.input_decimal();
    assert_eq!(c.display(), "0.");
    type_str(&mut c, "5");
    assert_eq!(c.display(), "0.5");
}

#[test]
fn second_decimal_point_is_ignored() {
    let mut c = Calculator::new();
    type_str(&mut c, "1.2");
    c.input_decimal();
    type_str(&mut c, "3");
    assert_eq!(c.display(), "1.23");
}

#[test]
fn decimal_point_is_ignored_at_digit_cap() {
    let mut c = Calculator::new();
    type_str(&mut c, "1234567890123456");
    c.input_decimal();
    assert_eq!(c.display(), "1,234,567,890,123,456");
}

#[test]
fn entry_length_is_capped() {
    let mut c = Calculator::new();
    type_str(&mut c, "12345678901234567890");
    assert_eq!(c.display(), "1,234,567,890,123,456");
}

#[test]
fn display_groups_thousands() {
    let mut c = Calculator::new();
    type_str(&mut c, "1234567");
    assert_eq!(c.display(), "1,234,567");
}

#[test]
fn addition() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    assert_eq!(c.display(), "5");
    assert_eq!(c.expression(), "2 + 3 =");
}

#[test]
fn subtraction() {
    let mut c = Calculator::new();
    type_str(&mut c, "9");
    c.set_op(Op::Subtract);
    type_str(&mut c, "4");
    c.equals();
    assert_eq!(c.display(), "5");
}

#[test]
fn multiplication() {
    let mut c = Calculator::new();
    type_str(&mut c, "6");
    c.set_op(Op::Multiply);
    type_str(&mut c, "7");
    c.equals();
    assert_eq!(c.display(), "42");
    assert_eq!(c.expression(), "6 × 7 =");
}

#[test]
fn division() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "2");
    c.equals();
    assert_eq!(c.display(), "2.5");
}

#[test]
fn floating_point_noise_is_hidden() {
    let mut c = Calculator::new();
    type_str(&mut c, "0.1");
    c.set_op(Op::Add);
    type_str(&mut c, "0.2");
    c.equals();
    assert_eq!(c.display(), "0.3");
}

#[test]
fn chained_operation_shows_intermediate_result() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.set_op(Op::Add);
    assert_eq!(c.display(), "5");
    type_str(&mut c, "4");
    c.equals();
    assert_eq!(c.display(), "9");
    assert_eq!(c.expression(), "5 + 4 =");
}

#[test]
fn equals_repeats_last_operation() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.equals();
    assert_eq!(c.display(), "8");
    c.equals();
    assert_eq!(c.display(), "11");
    assert_eq!(c.expression(), "8 + 3 =");
}

#[test]
fn equals_with_no_rhs_reuses_accumulator() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    c.equals();
    assert_eq!(c.display(), "4");
}

#[test]
fn operator_can_be_replaced_before_rhs() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    c.set_op(Op::Multiply);
    type_str(&mut c, "3");
    c.equals();
    assert_eq!(c.display(), "6");
    assert_eq!(c.expression(), "2 × 3 =");
}

#[test]
fn typing_after_equals_replaces_display() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    type_str(&mut c, "7");
    assert_eq!(c.display(), "7");
}

#[test]
fn equals_with_nothing_pending_is_a_no_op() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.equals();
    assert_eq!(c.display(), "5");
}

#[test]
fn equals_after_edit_does_not_repeat_last_operation() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.clear_entry();
    c.equals();
    assert_eq!(c.display(), "0");
}

#[test]
fn percent_adds_share_of_accumulator() {
    let mut c = Calculator::new();
    type_str(&mut c, "50");
    c.set_op(Op::Add);
    type_str(&mut c, "10");
    c.percent();
    assert_eq!(c.display(), "5");
    c.equals();
    assert_eq!(c.display(), "55");
}

#[test]
fn percent_subtracts_share_of_accumulator() {
    let mut c = Calculator::new();
    type_str(&mut c, "50");
    c.set_op(Op::Subtract);
    type_str(&mut c, "10");
    c.percent();
    c.equals();
    assert_eq!(c.display(), "45");
}

#[test]
fn percent_of_multiply_operand() {
    let mut c = Calculator::new();
    type_str(&mut c, "200");
    c.set_op(Op::Multiply);
    type_str(&mut c, "10");
    c.percent();
    c.equals();
    assert_eq!(c.display(), "20");
}

#[test]
fn percent_without_operator_divides_by_hundred() {
    let mut c = Calculator::new();
    type_str(&mut c, "50");
    c.percent();
    assert_eq!(c.display(), "0.5");
}

#[test]
fn toggle_sign_flips_entry() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.toggle_sign();
    assert_eq!(c.display(), "-5");
    c.toggle_sign();
    assert_eq!(c.display(), "5");
}

#[test]
fn toggle_sign_keeps_zero() {
    let mut c = Calculator::new();
    c.toggle_sign();
    assert_eq!(c.display(), "0");
}

#[test]
fn toggle_sign_on_result() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.toggle_sign();
    assert_eq!(c.display(), "-5");
    c.equals();
    assert_eq!(c.display(), "-5");
}

#[test]
fn backspace_removes_last_digit() {
    let mut c = Calculator::new();
    type_str(&mut c, "123");
    c.backspace();
    c.backspace();
    assert_eq!(c.display(), "1");
}

#[test]
fn backspace_down_to_zero() {
    let mut c = Calculator::new();
    type_str(&mut c, "1");
    c.backspace();
    assert_eq!(c.display(), "0");
}

#[test]
fn backspace_is_ignored_after_result() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.backspace();
    assert_eq!(c.display(), "5");
}

#[test]
fn clear_entry_keeps_pending_operation() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "5");
    c.clear_entry();
    c.equals();
    assert_eq!(c.display(), "2");
}

#[test]
fn clear_resets_everything_but_history() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.clear();
    assert_eq!(c.display(), "0");
    assert_eq!(c.expression(), "");
    assert_eq!(c.history().len(), 1);
}

#[test]
fn divide_by_zero_shows_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    assert_eq!(c.display(), "Cannot divide by zero");
    assert_eq!(c.error(), Some(CalcError::DivideByZero));
}

#[test]
fn operator_keys_are_ignored_during_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    c.set_op(Op::Add);
    assert_eq!(c.display(), "Cannot divide by zero");
    assert_eq!(c.error(), Some(CalcError::DivideByZero));
}

#[test]
fn digit_recovers_from_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    type_str(&mut c, "7");
    assert_eq!(c.display(), "7");
    assert_eq!(c.error(), None);
}

#[test]
fn clear_recovers_from_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    c.clear();
    assert_eq!(c.display(), "0");
    assert_eq!(c.error(), None);
}

#[test]
fn clear_entry_recovers_from_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    c.clear_entry();
    assert_eq!(c.display(), "0");
    assert_eq!(c.error(), None);
}

#[test]
fn backspace_recovers_from_error() {
    let mut c = Calculator::new();
    type_str(&mut c, "5");
    c.set_op(Op::Divide);
    type_str(&mut c, "0");
    c.equals();
    c.backspace();
    assert_eq!(c.display(), "0");
    assert_eq!(c.error(), None);
}

fn push_near_max(mut c: Calculator) -> Calculator {
    type_str(&mut c, "9999999999999999");
    c.set_op(Op::Multiply);
    type_str(&mut c, "9999999999999999");
    c.equals();
    for _ in 0..17 {
        c.equals();
    }
    c
}

#[test]
fn overflow_is_detected() {
    let mut c = push_near_max(Calculator::new());
    assert_eq!(c.error(), None);
    for _ in 0..20 {
        c.equals();
    }
    assert_eq!(c.error(), Some(CalcError::Overflow));
    assert_eq!(c.display(), "Overflow");
}

#[test]
fn percent_overflow_is_detected() {
    let mut c = push_near_max(Calculator::new());
    c.set_op(Op::Add);
    type_str(&mut c, "9999999999999999");
    c.percent();
    assert_eq!(c.error(), Some(CalcError::Overflow));
}

#[test]
fn equals_records_history() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    assert_eq!(c.history().len(), 1);
    let entry = &c.history()[0];
    assert_eq!(entry.expression, "2 + 3");
    assert_eq!(entry.result, "5");
    assert_eq!(entry.value, 5.0);
}

#[test]
fn history_groups_large_numbers() {
    let mut c = Calculator::new();
    type_str(&mut c, "1234567");
    c.set_op(Op::Add);
    type_str(&mut c, "1");
    c.equals();
    let entry = &c.history()[0];
    assert_eq!(entry.expression, "1,234,567 + 1");
    assert_eq!(entry.result, "1,234,568");
}

#[test]
fn history_is_capped() {
    let mut c = Calculator::new();
    for i in 0..105u32 {
        type_str(&mut c, "1");
        c.set_op(Op::Add);
        type_str(&mut c, &i.to_string());
        c.equals();
    }
    assert_eq!(c.history().len(), 100);
    assert_eq!(c.history()[0].expression, "1 + 5");
    assert_eq!(c.history()[99].expression, "1 + 104");
}

#[test]
fn clear_history_empties_it() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.clear_history();
    assert!(c.history().is_empty());
}

#[test]
fn recall_loads_result_for_further_calculation() {
    let mut c = Calculator::new();
    type_str(&mut c, "2");
    c.set_op(Op::Add);
    type_str(&mut c, "3");
    c.equals();
    c.clear();
    assert!(c.recall_history(0));
    assert_eq!(c.display(), "5");
    c.set_op(Op::Add);
    type_str(&mut c, "1");
    c.equals();
    assert_eq!(c.display(), "6");
}

#[test]
fn recall_out_of_bounds_returns_false() {
    let mut c = Calculator::new();
    assert!(!c.recall_history(0));
}
