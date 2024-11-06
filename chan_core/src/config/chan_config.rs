use std::collections::HashMap;
use crate::bi::bi_config::BiConfig;
use crate::buy_sell_point::bs_point_config::BSPointConfig;
use crate::common::{
    chan_exception::{ChanException, ErrCode},
    enums::TrendType,
    utils::parse_inf,
};
use crate::math::{
    boll::BollModel,
    demark::DemarkEngine,
    kdj::KDJ,
    macd::MACD,
    rsi::RSI,
    trend_model::TrendModel,
};
use crate::seg::seg_config::SegConfig;
use crate::zs::zs_config::ZSConfig;

/// Chan analysis configuration
#[derive(Debug, Clone)]
pub struct ChanConfig {
    pub bi_conf: BiConfig,
    pub seg_conf: SegConfig,
    pub zs_conf: ZSConfig,
    pub trigger_step: bool,
    pub skip_step: usize,
    pub kl_data_check: bool,
    pub max_kl_misalgin_cnt: usize,
    pub max_kl_inconsistent_cnt: usize,
    pub auto_skip_illegal_sub_lv: bool,
    pub print_warning: bool,
    pub print_err_time: bool,
    pub mean_metrics: Vec<i32>,
    pub trend_metrics: Vec<i32>,
    pub macd_config: HashMap<String, i32>,
    pub cal_demark: bool,
    pub cal_rsi: bool,
    pub cal_kdj: bool,
    pub rsi_cycle: i32,
    pub kdj_cycle: i32,
    pub demark_config: HashMap<String, bool>,
    pub boll_n: i32,
    pub bs_point_conf: BSPointConfig,
    pub seg_bs_point_conf: BSPointConfig,
}

impl ChanConfig {
    pub fn new(conf: Option<HashMap<String, serde_json::Value>>) -> Result<Self, ChanException> {
        let mut conf = ConfigWithCheck::new(conf.unwrap_or_default());

        let bi_conf = BiConfig::new(
            conf.get("bi_algo").unwrap_or("normal".to_string()),
            conf.get("bi_strict").unwrap_or(true),
            conf.get("bi_fx_check").unwrap_or("strict".to_string()),
            conf.get("gap_as_kl").unwrap_or(false),
            conf.get("bi_end_is_peak").unwrap_or(true),
            conf.get("bi_allow_sub_peak").unwrap_or(true),
        );

        let seg_conf = SegConfig::new(
            conf.get("seg_algo").unwrap_or("chan".to_string()),
            conf.get("left_seg_method").unwrap_or("peak".to_string()),
        );

        let zs_conf = ZSConfig::new(
            conf.get("zs_combine").unwrap_or(true),
            conf.get("zs_combine_mode").unwrap_or("zs".to_string()),
            conf.get("one_bi_zs").unwrap_or(false),
            conf.get("zs_algo").unwrap_or("normal".to_string()),
        );

        let mut macd_config = HashMap::new();
        macd_config.insert("fast".to_string(), 12);
        macd_config.insert("slow".to_string(), 26);
        macd_config.insert("signal".to_string(), 9);

        let mut demark_config = HashMap::new();
        demark_config.insert("demark_len".to_string(), 9);
        demark_config.insert("setup_bias".to_string(), 4);
        demark_config.insert("countdown_bias".to_string(), 2);
        demark_config.insert("max_countdown".to_string(), 13);
        demark_config.insert("tiaokong_st".to_string(), true);
        demark_config.insert("setup_cmp2close".to_string(), true);
        demark_config.insert("countdown_cmp2close".to_string(), true);

        let mut config = Self {
            bi_conf,
            seg_conf,
            zs_conf,
            trigger_step: conf.get("trigger_step").unwrap_or(false),
            skip_step: conf.get("skip_step").unwrap_or(0),
            kl_data_check: conf.get("kl_data_check").unwrap_or(true),
            max_kl_misalgin_cnt: conf.get("max_kl_misalgin_cnt").unwrap_or(2),
            max_kl_inconsistent_cnt: conf.get("max_kl_inconsistent_cnt").unwrap_or(5),
            auto_skip_illegal_sub_lv: conf.get("auto_skip_illegal_sub_lv").unwrap_or(false),
            print_warning: conf.get("print_warning").unwrap_or(true),
            print_err_time: conf.get("print_err_time").unwrap_or(false),
            mean_metrics: conf.get("mean_metrics").unwrap_or_else(Vec::new),
            trend_metrics: conf.get("trend_metrics").unwrap_or_else(Vec::new),
            macd_config: conf.get("macd").unwrap_or(macd_config),
            cal_demark: conf.get("cal_demark").unwrap_or(false),
            cal_rsi: conf.get("cal_rsi").unwrap_or(false),
            cal_kdj: conf.get("cal_kdj").unwrap_or(false),
            rsi_cycle: conf.get("rsi_cycle").unwrap_or(14),
            kdj_cycle: conf.get("kdj_cycle").unwrap_or(9),
            demark_config: conf.get("demark").unwrap_or(demark_config),
            boll_n: conf.get("boll_n").unwrap_or(20),
            bs_point_conf: BSPointConfig::default(),
            seg_bs_point_conf: BSPointConfig::default(),
        };

        config.set_bsp_config(&mut conf)?;
        conf.check()?;

        Ok(config)
    }

    pub fn get_metric_model(&self) -> Vec<Box<dyn MetricModel>> {
        let mut res: Vec<Box<dyn MetricModel>> = Vec::new();
        
        // Add MACD
        res.push(Box::new(MACD::new(
            self.macd_config["fast"],
            self.macd_config["slow"],
            self.macd_config["signal"],
        )));

        // Add mean trend models
        for mean_t in &self.mean_metrics {
            res.push(Box::new(TrendModel::new(TrendType::Mean, *mean_t)));
        }

        // Add max/min trend models
        for trend_t in &self.trend_metrics {
            res.push(Box::new(TrendModel::new(TrendType::Max, *trend_t)));
            res.push(Box::new(TrendModel::new(TrendType::Min, *trend_t)));
        }

        // Add BOLL model
        res.push(Box::new(BollModel::new(self.boll_n)));

        // Add Demark if enabled
        if self.cal_demark {
            res.push(Box::new(DemarkEngine::new(
                self.demark_config["demark_len"],
                self.demark_config["setup_bias"],
                self.demark_config["countdown_bias"],
                self.demark_config["max_countdown"],
                self.demark_config["tiaokong_st"],
                self.demark_config["setup_cmp2close"],
                self.demark_config["countdown_cmp2close"],
            )));
        }

        // Add RSI if enabled
        if self.cal_rsi {
            res.push(Box::new(RSI::new(self.rsi_cycle)));
        }

        // Add KDJ if enabled
        if self.cal_kdj {
            res.push(Box::new(KDJ::new(self.kdj_cycle)));
        }

        res
    }

    fn set_bsp_config(&mut self, conf: &mut ConfigWithCheck) -> Result<(), ChanException> {
        let mut para_dict = HashMap::new();
        para_dict.insert("divergence_rate", serde_json::Value::from(f64::INFINITY));
        para_dict.insert("min_zs_cnt", serde_json::Value::from(1));
        para_dict.insert("bsp1_only_multibi_zs", serde_json::Value::from(true));
        para_dict.insert("max_bs2_rate", serde_json::Value::from(0.9999));
        para_dict.insert("macd_algo", serde_json::Value::from("peak"));
        para_dict.insert("bs1_peak", serde_json::Value::from(true));
        para_dict.insert("bs_type", serde_json::Value::from("1,1p,2,2s,3a,3b"));
        para_dict.insert("bsp2_follow_1", serde_json::Value::from(true));
        para_dict.insert("bsp3_follow_1", serde_json::Value::from(true));
        para_dict.insert("bsp3_peak", serde_json::Value::from(false));
        para_dict.insert("bsp2s_follow_2", serde_json::Value::from(false));
        para_dict.insert("max_bsp2s_lv", serde_json::Value::Null);
        para_dict.insert("strict_bsp3", serde_json::Value::from(false));

        let args: HashMap<String, serde_json::Value> = para_dict.iter()
            .map(|(k, v)| (k.to_string(), conf.get(k).unwrap_or_else(|| v.clone())))
            .collect();

        self.bs_point_conf = BSPointConfig::from_args(&args)?;
        self.seg_bs_point_conf = BSPointConfig::from_args(&args)?;

        // Set specific configurations for seg_bs_point_conf
        self.seg_bs_point_conf.b_conf.set("macd_algo", "slope")?;
        self.seg_bs_point_conf.s_conf.set("macd_algo", "slope")?;
        self.seg_bs_point_conf.b_conf.set("bsp1_only_multibi_zs", false)?;
        self.seg_bs_point_conf.s_conf.set("bsp1_only_multibi_zs", false)?;

        // Process remaining configurations
        for (k, v) in conf.iter() {
            let v_str = if v.is_string() {
                format!("\"{}\"", v.as_str().unwrap())
            } else {
                parse_inf(&v.to_string())
            };

            if k.ends_with("-buy") {
                let prop = k.replace("-buy", "");
                self.bs_point_conf.b_conf.set(&prop, v_str)?;
            } else if k.ends_with("-sell") {
                let prop = k.replace("-sell", "");
                self.bs_point_conf.s_conf.set(&prop, v_str)?;
            } else if k.ends_with("-segbuy") {
                let prop = k.replace("-segbuy", "");
                self.seg_bs_point_conf.b_conf.set(&prop, v_str)?;
            } else if k.ends_with("-segsell") {
                let prop = k.replace("-segsell", "");
                self.seg_bs_point_conf.s_conf.set(&prop, v_str)?;
            } else if k.ends_with("-seg") {
                let prop = k.replace("-seg", "");
                self.seg_bs_point_conf.b_conf.set(&prop, v_str)?;
                self.seg_bs_point_conf.s_conf.set(&prop, v_str)?;
            } else if args.contains_key(k) {
                self.bs_point_conf.b_conf.set(k, v_str.clone())?;
                self.bs_point_conf.s_conf.set(k, v_str)?;
            } else {
                return Err(ChanException::new(
                    format!("unknown para = {}", k),
                    ErrCode::ParaError,
                ));
            }
        }

        // Parse target types
        self.bs_point_conf.b_conf.parse_target_type()?;
        self.bs_point_conf.s_conf.parse_target_type()?;
        self.seg_bs_point_conf.b_conf.parse_target_type()?;
        self.seg_bs_point_conf.s_conf.parse_target_type()?;

        Ok(())
    }
} 