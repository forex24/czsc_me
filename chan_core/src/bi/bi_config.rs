use crate::common::{
    enums::FxCheckMethod,
    chan_exception::{ChanException, ErrCode},
};

/// 笔的配置结构
#[derive(Debug, Clone)]
pub struct BiConfig {
    /// 笔的算法："normal" 或 "fx"
    pub bi_algo: String,
    
    /// 是否使用严格模式
    pub is_strict: bool,
    
    /// 分型检查方法
    pub bi_fx_check: FxCheckMethod,
    
    /// 是否将跳空视为一根K线
    pub gap_as_kl: bool,
    
    /// 笔的端点是否必须是峰值
    pub bi_end_is_peak: bool,
    
    /// 是否允许次级别的峰值
    pub bi_allow_sub_peak: bool,
}

impl BiConfig {
    /// Create a new BiConfig with default values
    pub fn new(
        bi_algo: Option<String>,
        is_strict: Option<bool>,
        bi_fx_check: Option<&str>,
        gap_as_kl: Option<bool>,
        bi_end_is_peak: Option<bool>,
        bi_allow_sub_peak: Option<bool>,
    ) -> Result<Self, ChanException> {
        let bi_fx_check = match bi_fx_check.unwrap_or("half") {
            "strict" => FxCheckMethod::Strict,
            "loss" => FxCheckMethod::Loss,
            "half" => FxCheckMethod::Half,
            "totally" => FxCheckMethod::Totally,
            unknown => return Err(ChanException::new(
                format!("unknown bi_fx_check={}", unknown),
                ErrCode::ParaError,
            )),
        };

        Ok(Self {
            bi_algo: bi_algo.unwrap_or_else(|| "normal".to_string()),
            is_strict: is_strict.unwrap_or(true),
            bi_fx_check,
            gap_as_kl: gap_as_kl.unwrap_or(true),
            bi_end_is_peak: bi_end_is_peak.unwrap_or(true),
            bi_allow_sub_peak: bi_allow_sub_peak.unwrap_or(true),
        })
    }

    /// Create a default BiConfig
    pub fn default() -> Self {
        Self::new(None, None, None, None, None, None)
            .expect("Default BiConfig creation should never fail")
    }
}

impl Default for BiConfig {
    fn default() -> Self {
        Self::default()
    }
} 