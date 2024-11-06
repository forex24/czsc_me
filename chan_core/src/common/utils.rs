use super::enums::{BiDir, KlType};
use crate::common::chan_exception::{ChanException, ErrCode};

/// Check if kline type is less than day
pub fn kltype_lt_day(ktype: KlType) -> bool {
    ktype as i32 < KlType::KDay as i32
}

/// Check if kline type is less than or equal to day
pub fn kltype_lte_day(ktype: KlType) -> bool {
    ktype as i32 <= KlType::KDay as i32
}

/// Check if kline type list is ordered from large to small
pub fn check_kltype_order(type_list: &[KlType]) -> Result<(), ChanException> {
    let mut last_lv = type_list[0] as i32;
    for &kl_type in &type_list[1..] {
        if kl_type as i32 >= last_lv {
            return Err(ChanException::new(
                "lv_list的顺序必须从大级别到小级别".to_string(),
                ErrCode::ParaError,
            ));
        }
        last_lv = kl_type as i32;
    }
    Ok(())
}

/// Revert bi direction
pub fn revert_bi_dir(dir: BiDir) -> BiDir {
    match dir {
        BiDir::Up => BiDir::Down,
        BiDir::Down => BiDir::Up,
    }
}

/// Check if two ranges have overlap
pub fn has_overlap(l1: f64, h1: f64, l2: f64, h2: f64, equal: bool) -> bool {
    if equal {
        h2 >= l1 && h1 >= l2
    } else {
        h2 > l1 && h1 > l2
    }
}

/// Convert string to float
pub fn str2float(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

/// Parse infinity values in configuration
pub fn parse_inf(v: &str) -> String {
    match v {
        "inf" | "float(\"inf\")" => "f64::INFINITY".to_string(),
        "-inf" | "float(\"-inf\")" => "f64::NEG_INFINITY".to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kltype_lt_day() {
        assert!(kltype_lt_day(KlType::K1));
        assert!(kltype_lt_day(KlType::K5));
        assert!(!kltype_lt_day(KlType::KDay));
        assert!(!kltype_lt_day(KlType::KWeek));
    }

    #[test]
    fn test_kltype_lte_day() {
        assert!(kltype_lte_day(KlType::K1));
        assert!(kltype_lte_day(KlType::K5));
        assert!(kltype_lte_day(KlType::KDay));
        assert!(!kltype_lte_day(KlType::KWeek));
    }

    #[test]
    fn test_revert_bi_dir() {
        assert_eq!(revert_bi_dir(BiDir::Up), BiDir::Down);
        assert_eq!(revert_bi_dir(BiDir::Down), BiDir::Up);
    }

    #[test]
    fn test_has_overlap() {
        assert!(has_overlap(1.0, 3.0, 2.0, 4.0, false));
        assert!(has_overlap(2.0, 4.0, 1.0, 3.0, false));
        assert!(!has_overlap(1.0, 2.0, 3.0, 4.0, false));
        assert!(has_overlap(1.0, 2.0, 2.0, 3.0, true));
        assert!(!has_overlap(1.0, 2.0, 2.0, 3.0, false));
    }

    #[test]
    fn test_str2float() {
        assert_eq!(str2float("1.23"), 1.23);
        assert_eq!(str2float("invalid"), 0.0);
    }

    #[test]
    fn test_parse_inf() {
        assert_eq!(parse_inf("inf"), "f64::INFINITY");
        assert_eq!(parse_inf("-inf"), "f64::NEG_INFINITY");
        assert_eq!(parse_inf("1.23"), "1.23");
    }
} 