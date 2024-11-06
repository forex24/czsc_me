use std::collections::HashMap;
use strum_macros::{Display, EnumString};
use crate::common::enums::BspType;
use crate::common::utils::parse_inf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum MacdAlgo {
    #[strum(serialize = "area")]
    Area,
    #[strum(serialize = "peak")]
    Peak,
    #[strum(serialize = "full_area")]
    FullArea,
    #[strum(serialize = "diff")]
    Diff,
    #[strum(serialize = "slope")]
    Slope,
    #[strum(serialize = "amp")]
    Amp,
    #[strum(serialize = "amount")]
    Amount,
    #[strum(serialize = "volumn")]
    Volumn,
    #[strum(serialize = "amount_avg")]
    AmountAvg,
    #[strum(serialize = "volumn_avg")]
    VolumnAvg,
    #[strum(serialize = "turnrate_avg")]
    TurnrateAvg,
    #[strum(serialize = "rsi")]
    Rsi,
}

#[derive(Debug, Clone)]
pub struct BSPointConfig {
    pub b_conf: PointConfig,
    pub s_conf: PointConfig,
}

impl BSPointConfig {
    pub fn new(args: HashMap<String, String>) -> Self {
        let b_conf = PointConfig::new(&args);
        let s_conf = PointConfig::new(&args);
        Self { b_conf, s_conf }
    }

    pub fn get_bs_config(&self, is_buy: bool) -> &PointConfig {
        if is_buy { &self.b_conf } else { &self.s_conf }
    }
}

#[derive(Debug, Clone)]
pub struct PointConfig {
    pub divergence_rate: f64,
    pub min_zs_cnt: i32,
    pub bsp1_only_multibi_zs: bool,
    pub max_bs2_rate: f64,
    pub macd_algo: MacdAlgo,
    pub bs1_peak: bool,
    pub tmp_target_types: Vec<String>,
    pub target_types: Vec<BspType>,
    pub bsp2_follow_1: bool,
    pub bsp3_follow_1: bool,
    pub bsp3_peak: bool,
    pub bsp2s_follow_2: bool,
    pub max_bsp2s_lv: Option<i32>,
    pub strict_bsp3: bool,
}

impl PointConfig {
    pub fn new(args: &HashMap<String, String>) -> Self {
        let mut config = Self {
            divergence_rate: args.get("divergence_rate").and_then(|v| v.parse().ok()).unwrap_or(0.0),
            min_zs_cnt: args.get("min_zs_cnt").and_then(|v| v.parse().ok()).unwrap_or(0),
            bsp1_only_multibi_zs: args.get("bsp1_only_multibi_zs").and_then(|v| v.parse().ok()).unwrap_or(false),
            max_bs2_rate: args.get("max_bs2_rate").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            macd_algo: MacdAlgo::Area,
            bs1_peak: args.get("bs1_peak").and_then(|v| v.parse().ok()).unwrap_or(false),
            tmp_target_types: vec![],
            target_types: vec![],
            bsp2_follow_1: args.get("bsp2_follow_1").and_then(|v| v.parse().ok()).unwrap_or(false),
            bsp3_follow_1: args.get("bsp3_follow_1").and_then(|v| v.parse().ok()).unwrap_or(false),
            bsp3_peak: args.get("bsp3_peak").and_then(|v| v.parse().ok()).unwrap_or(false),
            bsp2s_follow_2: args.get("bsp2s_follow_2").and_then(|v| v.parse().ok()).unwrap_or(false),
            max_bsp2s_lv: args.get("max_bsp2s_lv").and_then(|v| v.parse().ok()),
            strict_bsp3: args.get("strict_bsp3").and_then(|v| v.parse().ok()).unwrap_or(false),
        };

        if let Some(macd_algo) = args.get("macd_algo") {
            config.set_macd_algo(macd_algo);
        }

        if let Some(bs_type) = args.get("bs_type") {
            config.tmp_target_types = bs_type.split(',').map(|s| s.trim().to_string()).collect();
            config.parse_target_type();
        }

        assert!(config.max_bs2_rate <= 1.0);
        config
    }

    pub fn parse_target_type(&mut self) {
        let valid_types = ["1", "2", "3a", "2s", "1p", "3b"];
        for target_t in &self.tmp_target_types {
            assert!(valid_types.contains(&target_t.as_str()));
        }

        self.target_types = self.tmp_target_types.iter()
            .map(|t| match t.as_str() {
                "1" => BspType::BS1,
                "2" => BspType::BS2,
                "3a" => BspType::BS3,
                "2s" => BspType::BS2Strict,
                "1p" => BspType::BS1Peak,
                "3b" => BspType::BS3Peak,
                _ => unreachable!(),
            })
            .collect();
    }

    pub fn set_macd_algo(&mut self, algo: &str) {
        self.macd_algo = match algo {
            "area" => MacdAlgo::Area,
            "peak" => MacdAlgo::Peak,
            "full_area" => MacdAlgo::FullArea,
            "diff" => MacdAlgo::Diff,
            "slope" => MacdAlgo::Slope,
            "amp" => MacdAlgo::Amp,
            "amount" => MacdAlgo::Amount,
            "volumn" => MacdAlgo::Volumn,
            "amount_avg" => MacdAlgo::AmountAvg,
            "volumn_avg" => MacdAlgo::VolumnAvg,
            "turnrate_avg" => MacdAlgo::TurnrateAvg,
            "rsi" => MacdAlgo::Rsi,
            _ => panic!("Unknown MACD algorithm: {}", algo),
        };
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let value = parse_inf(value);
        match key {
            "macd_algo" => self.set_macd_algo(value),
            "divergence_rate" => self.divergence_rate = value.parse().unwrap(),
            "min_zs_cnt" => self.min_zs_cnt = value.parse().unwrap(),
            "bsp1_only_multibi_zs" => self.bsp1_only_multibi_zs = value.parse().unwrap(),
            "max_bs2_rate" => self.max_bs2_rate = value.parse().unwrap(),
            "bs1_peak" => self.bs1_peak = value.parse().unwrap(),
            "bsp2_follow_1" => self.bsp2_follow_1 = value.parse().unwrap(),
            "bsp3_follow_1" => self.bsp3_follow_1 = value.parse().unwrap(),
            "bsp3_peak" => self.bsp3_peak = value.parse().unwrap(),
            "bsp2s_follow_2" => self.bsp2s_follow_2 = value.parse().unwrap(),
            "max_bsp2s_lv" => self.max_bsp2s_lv = Some(value.parse().unwrap()),
            "strict_bsp3" => self.strict_bsp3 = value.parse().unwrap(),
            _ => panic!("Unknown config key: {}", key),
        }
    }
} 