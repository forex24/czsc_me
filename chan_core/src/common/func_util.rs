use crate::common::enums::{BiDir, KlType};
use std::f64;

/// Check if KL type is less than day
pub fn kltype_lt_day(kl_type: KlType) -> bool {
    (kl_type as i32) < (KlType::KDay as i32)
}

/// Check if KL type is less than or equal to day
pub fn kltype_lte_day(kl_type: KlType) -> bool {
    (kl_type as i32) <= (KlType::KDay as i32)
}

/// Check if KL type list is ordered from large to small
pub fn check_kltype_order(type_list: &[KlType]) -> Result<(), String> {
    if type_list.is_empty() {
        return Ok(());
    }

    let mut last_lv = type_list[0] as i32;
    for &kl_type in &type_list[1..] {
        if (kl_type as i32) >= last_lv {
            return Err("lv_list的顺序必须从大级别到小级别".to_string());
        }
        last_lv = kl_type as i32;
    }
    Ok(())
}

/// Revert BiDir direction
pub fn revert_bi_dir(dir: BiDir) -> BiDir {
    match dir {
        BiDir::Up => BiDir::Down,
        BiDir::Down => BiDir::Up,
    }
}

/// Check if two ranges overlap
pub fn has_overlap(l1: f64, h1: f64, l2: f64, h2: f64, equal: bool) -> bool {
    if equal {
        h2 >= l1 && h1 >= l2
    } else {
        h2 > l1 && h1 > l2
    }
}

/// Convert string to float, returns 0.0 if conversion fails
pub fn str2float(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

/// Parse infinity values to string representation
pub fn parse_inf(v: f64) -> String {
    if v.is_infinite() {
        if v.is_sign_positive() {
            r#"float("inf")"#.to_string()
        } else {
            r#"float("-inf")"#.to_string()
        }
    } else {
        v.to_string()
    }
} 