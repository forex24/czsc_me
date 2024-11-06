use crate::common::enums::BiDir;
use crate::common::handle::Handle;
use crate::kline::kline_unit::KLineUnit;

pub trait LineTrait {
    /// Get the beginning KLineUnit
    fn get_begin_klu(&self) -> Handle<KLineUnit>;
    
    /// Get the ending KLineUnit
    fn get_end_klu(&self) -> Handle<KLineUnit>;
    
    /// Get the low value
    fn _low(&self) -> f64;
    
    /// Get the high value
    fn _high(&self) -> f64;
    
    /// Get the index
    fn idx(&self) -> usize;
    
    /// Get the segment index
    fn seg_idx(&self) -> usize;
    
    /// Get the end value
    fn get_end_val(&self) -> f64;
    
    /// Check if it's a downward direction
    fn is_down(&self) -> bool;
    
    /// Check if it's an upward direction
    fn is_up(&self) -> bool;
    
    /// Calculate MACD metric
    fn cal_macd_metric(&self, algo: &str, is_reverse: bool) -> f64;
} 