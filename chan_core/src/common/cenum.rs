use strum_macros::{Display, EnumString};

/// Data source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DataSrc {
    BaoStock,
    Ccxt,
    Csv,
}

/// Kline time period types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum KlType {
    #[strum(serialize = "K_1S")]
    K1S = 1,
    #[strum(serialize = "K_3S")]
    K3S = 2,
    #[strum(serialize = "K_5S")]
    K5S = 3,
    #[strum(serialize = "K_10S")]
    K10S = 4,
    #[strum(serialize = "K_15S")]
    K15S = 5,
    #[strum(serialize = "K_20S")]
    K20S = 6,
    #[strum(serialize = "K_30S")]
    K30S = 7,
    #[strum(serialize = "K_1M")]
    K1M = 8,
    #[strum(serialize = "K_3M")]
    K3M = 9,
    #[strum(serialize = "K_5M")]
    K5M = 10,
    #[strum(serialize = "K_10M")]
    K10M = 11,
    #[strum(serialize = "K_15M")]
    K15M = 12,
    #[strum(serialize = "K_30M")]
    K30M = 13,
    #[strum(serialize = "K_60M")]
    K60M = 14,
    #[strum(serialize = "K_DAY")]
    KDay = 15,
    #[strum(serialize = "K_WEEK")]
    KWeek = 16,
    #[strum(serialize = "K_MON")]
    KMon = 17,
    #[strum(serialize = "K_QUARTER")]
    KQuarter = 18,
    #[strum(serialize = "K_YEAR")]
    KYear = 19,
}

/// Kline direction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum KlineDir {
    Up,
    Down,
    Combine,
    Included,
}

/// Fractal types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FxType {
    Bottom,
    Top,
    Unknown,
}

/// BI direction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum BiDir {
    Up,
    Down,
}

/// BI analysis types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum BiType {
    Unknown,
    Strict,
    SubValue, // 次高低点成笔
    TiaokongThred,
    Daheng,
    Tuibi,
    Unstrict,
    TiaokongValue,
}

/// BSP type with string values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum BspType {
    #[strum(serialize = "1")]
    T1,
    #[strum(serialize = "1p")]
    T1P,
    #[strum(serialize = "2")]
    T2,
    #[strum(serialize = "2s")]
    T2S,
    #[strum(serialize = "3a")]
    T3A, // 中枢在1类后面
    #[strum(serialize = "3b")]
    T3B, // 中枢在1类前面
}

impl BspType {
    pub fn main_type(&self) -> &'static str {
        match self {
            Self::T1 | Self::T1P => "1",
            Self::T2 | Self::T2S => "2",
            Self::T3A | Self::T3B => "3",
        }
    }
}

/// Adjustment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AuType {
    Qfq,
    Hfq,
    None,
}

/// Trend analysis types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum TrendType {
    #[strum(serialize = "mean")]
    Mean,
    #[strum(serialize = "max")]
    Max,
    #[strum(serialize = "min")]
    Min,
}

/// Trend line side types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TrendLineSide {
    Inside,
    Outside,
}

/// Left segment analysis methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum LeftSegMethod {
    All,
    Peak,
}

/// FX check methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FxCheckMethod {
    Strict,
    Loss,
    Half,
    Totally,
}

/// Segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SegType {
    Bi,
    Seg,
}

/// MACD calculation algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MacdAlgo {
    Area,
    Peak,
    FullArea,
    Diff,
    Slope,
    Amp,
    Volumn,
    Amount,
    VolumnAvg,
    AmountAvg,
    TurnrateAvg,
    Rsi,
}

/// Data field constants
#[derive(Debug, Clone)]
pub struct DataField;

impl DataField {
    pub const FIELD_TIME: &'static str = "time_key";
    pub const FIELD_OPEN: &'static str = "open";
    pub const FIELD_HIGH: &'static str = "high";
    pub const FIELD_LOW: &'static str = "low";
    pub const FIELD_CLOSE: &'static str = "close";
    pub const FIELD_VOLUME: &'static str = "volume"; // 成交量
    pub const FIELD_TURNOVER: &'static str = "turnover"; // 成交额
    pub const FIELD_TURNRATE: &'static str = "turnover_rate"; // 换手率
}

pub const TRADE_INFO_LST: &[&str] = &[
    DataField::FIELD_VOLUME,
    DataField::FIELD_TURNOVER,
    DataField::FIELD_TURNRATE,
];
