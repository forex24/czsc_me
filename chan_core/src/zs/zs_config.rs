use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Display, EnumString)]
pub enum ZSAlgo {
    #[strum(serialize = "normal")]
    Normal,
}

#[derive(Debug, Clone)]
pub struct ZSConfig {
    /// Whether combination is needed
    pub need_combine: bool,
    
    /// Mode for ZS combination ("zs" or "peak")
    pub zs_combine_mode: String,
    
    /// Whether to allow single bi ZS
    pub one_bi_zs: bool,
    
    /// Algorithm for ZS calculation
    pub zs_algo: ZSAlgo,
}

impl Default for ZSConfig {
    fn default() -> Self {
        Self {
            need_combine: true,
            zs_combine_mode: "zs".to_string(),
            one_bi_zs: false,
            zs_algo: ZSAlgo::Normal,
        }
    }
}

impl ZSConfig {
    /// Create a new ZSConfig with custom parameters
    pub fn new(
        need_combine: Option<bool>,
        zs_combine_mode: Option<&str>,
        one_bi_zs: Option<bool>,
        zs_algo: Option<ZSAlgo>,
    ) -> Self {
        let default = Self::default();
        Self {
            need_combine: need_combine.unwrap_or(default.need_combine),
            zs_combine_mode: zs_combine_mode.unwrap_or(&default.zs_combine_mode).to_string(),
            one_bi_zs: one_bi_zs.unwrap_or(default.one_bi_zs),
            zs_algo: zs_algo.unwrap_or(default.zs_algo),
        }
    }
} 