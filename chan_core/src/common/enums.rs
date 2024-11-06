#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum FxCheckMethod {
    Strict,
    Loss,
    Half,
    Totally,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
pub enum BiDir {
    Up,
    Down,
}

use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
pub enum BspType {
    #[strum(serialize = "BS1")]
    BS1,
    #[strum(serialize = "BS2")]
    BS2,
    #[strum(serialize = "BS3")]
    BS3,
    #[strum(serialize = "BS4")]
    BS4,
    #[strum(serialize = "BS5")]
    BS5,
    #[strum(serialize = "BS1_PEAK")]
    BS1Peak,
    #[strum(serialize = "BS2_PEAK")]
    BS2Peak,
    #[strum(serialize = "BS3_PEAK")]
    BS3Peak,
    #[strum(serialize = "BS4_PEAK")]
    BS4Peak,
    #[strum(serialize = "BS5_PEAK")]
    BS5Peak,
    #[strum(serialize = "BS1_STRICT")]
    BS1Strict,
    #[strum(serialize = "BS2_STRICT")]
    BS2Strict,
    #[strum(serialize = "BS3_STRICT")]
    BS3Strict,
    #[strum(serialize = "BS4_STRICT")]
    BS4Strict,
    #[strum(serialize = "BS5_STRICT")]
    BS5Strict,
    #[strum(serialize = "BS1_PEAK_STRICT")]
    BS1PeakStrict,
    #[strum(serialize = "BS2_PEAK_STRICT")]
    BS2PeakStrict,
    #[strum(serialize = "BS3_PEAK_STRICT")]
    BS3PeakStrict,
    #[strum(serialize = "BS4_PEAK_STRICT")]
    BS4PeakStrict,
    #[strum(serialize = "BS5_PEAK_STRICT")]
    BS5PeakStrict,
}

impl BspType {
    /// Get the base type (removing PEAK and STRICT modifiers)
    pub fn base_type(&self) -> Self {
        match self {
            BspType::BS1 | BspType::BS1Peak | BspType::BS1Strict | BspType::BS1PeakStrict => {
                BspType::BS1
            }
            BspType::BS2 | BspType::BS2Peak | BspType::BS2Strict | BspType::BS2PeakStrict => {
                BspType::BS2
            }
            BspType::BS3 | BspType::BS3Peak | BspType::BS3Strict | BspType::BS3PeakStrict => {
                BspType::BS3
            }
            BspType::BS4 | BspType::BS4Peak | BspType::BS4Strict | BspType::BS4PeakStrict => {
                BspType::BS4
            }
            BspType::BS5 | BspType::BS5Peak | BspType::BS5Strict | BspType::BS5PeakStrict => {
                BspType::BS5
            }
        }
    }

    /// Check if this is a peak type
    pub fn is_peak(&self) -> bool {
        matches!(
            self,
            BspType::BS1Peak
                | BspType::BS2Peak
                | BspType::BS3Peak
                | BspType::BS4Peak
                | BspType::BS5Peak
                | BspType::BS1PeakStrict
                | BspType::BS2PeakStrict
                | BspType::BS3PeakStrict
                | BspType::BS4PeakStrict
                | BspType::BS5PeakStrict
        )
    }

    /// Check if this is a strict type
    pub fn is_strict(&self) -> bool {
        matches!(
            self,
            BspType::BS1Strict
                | BspType::BS2Strict
                | BspType::BS3Strict
                | BspType::BS4Strict
                | BspType::BS5Strict
                | BspType::BS1PeakStrict
                | BspType::BS2PeakStrict
                | BspType::BS3PeakStrict
                | BspType::BS4PeakStrict
                | BspType::BS5PeakStrict
        )
    }
}
