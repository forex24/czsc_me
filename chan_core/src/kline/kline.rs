use std::ops::{Index, IndexMut};
use crate::common::{
    handle::Handle,
    enums::{FxType, KlineDir},
    chan_exception::{ChanException, ErrCode},
    time::Time,
};
use crate::kline::kline_unit::KLineUnit;
use crate::impl_handle;

/// Represents a combined K-line structure
#[derive(Debug)]
pub struct KLine {
    handle: Handle<Self>,
    pub idx: usize,
    pub kl_type: Option<String>,
    
    // Internal state from KLineCombiner
    time_begin: Time,
    time_end: Time,
    high: f64,
    low: f64,
    lst: Vec<KLineUnit>,
    dir: KlineDir,
    fx: FxType,
}

impl KLine {
    pub fn new(kl_unit: &mut KLineUnit, box_vec: &Box<Vec<Self>>, index: usize, dir: KlineDir) -> Self {
        let mut kline = Self {
            handle: Handle::new(box_vec, index),
            idx: index,
            kl_type: kl_unit.kl_type.clone(),
            time_begin: kl_unit.time,
            time_end: kl_unit.time,
            high: kl_unit.high,
            low: kl_unit.low,
            lst: vec![kl_unit.clone()],
            dir,
            fx: FxType::Unknown,
        };
        kl_unit.set_klc(&kline);
        kline
    }

    // Getters
    pub fn time_begin(&self) -> Time { self.time_begin }
    pub fn time_end(&self) -> Time { self.time_end }
    pub fn high(&self) -> f64 { self.high }
    pub fn low(&self) -> f64 { self.low }
    pub fn dir(&self) -> KlineDir { self.dir }
    pub fn fx(&self) -> FxType { self.fx }

    /// Test if this KLine can combine with another item
    pub fn test_combine(&self, item_high: f64, item_low: f64, exclude_included: bool, allow_top_equal: Option<i32>) -> Result<KlineDir, ChanException> {
        if self.high >= item_high && self.low <= item_low {
            return Ok(KlineDir::Combine);
        }
        if self.high <= item_high && self.low >= item_low {
            match allow_top_equal {
                Some(1) if self.high == item_high && self.low > item_low => return Ok(KlineDir::Down),
                Some(-1) if self.low == item_low && self.high < item_high => return Ok(KlineDir::Up),
                _ => return Ok(if exclude_included { KlineDir::Included } else { KlineDir::Combine }),
            }
        }
        if self.high > item_high && self.low > item_low {
            return Ok(KlineDir::Down);
        }
        if self.high < item_high && self.low < item_low {
            return Ok(KlineDir::Up);
        }
        Err(ChanException::new("combine type unknown".into(), ErrCode::CombinerErr))
    }

    /// Try to add a new KLineUnit
    pub fn try_add(&mut self, unit_kl: &mut KLineUnit, exclude_included: bool, allow_top_equal: Option<i32>) -> Result<KlineDir, ChanException> {
        let dir = self.test_combine(unit_kl.high, unit_kl.low, exclude_included, allow_top_equal)?;
        
        if dir == KlineDir::Combine {
            self.lst.push(unit_kl.clone());
            unit_kl.set_klc(self);

            match self.dir {
                KlineDir::Up => {
                    if unit_kl.high != unit_kl.low || unit_kl.high != self.high {
                        self.high = self.high.max(unit_kl.high);
                        self.low = self.low.max(unit_kl.low);
                    }
                }
                KlineDir::Down => {
                    if unit_kl.high != unit_kl.low || unit_kl.low != self.low {
                        self.high = self.high.min(unit_kl.high);
                        self.low = self.low.min(unit_kl.low);
                    }
                }
                _ => return Err(ChanException::new(
                    format!("KLINE_DIR = {:?} err!!! must be Up/Down", self.dir),
                    ErrCode::CombinerErr
                )),
            }
            self.time_end = unit_kl.time;
        }
        
        Ok(dir)
    }

    /// Get peak KLineUnit
    pub fn get_peak_klu(&self, is_high: bool) -> Result<&KLineUnit, ChanException> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    fn get_high_peak_klu(&self) -> Result<&KLineUnit, ChanException> {
        self.lst.iter().rev()
            .find(|kl| kl.high == self.high)
            .ok_or_else(|| ChanException::new("can't find peak...".into(), ErrCode::CombinerErr))
    }

    fn get_low_peak_klu(&self) -> Result<&KLineUnit, ChanException> {
        self.lst.iter().rev()
            .find(|kl| kl.low == self.low)
            .ok_or_else(|| ChanException::new("can't find peak...".into(), ErrCode::CombinerErr))
    }

    /// Update FX (分型) status
    pub fn update_fx(&mut self, exclude_included: bool, allow_top_equal: Option<i32>) {
        let pre = self.prev().expect("pre should exist");
        let next = self.next().expect("next should exist");

        if exclude_included {
            if pre.high < self.high && next.high <= self.high && next.low < self.low {
                if allow_top_equal == Some(1) || next.high < self.high {
                    self.fx = FxType::Top;
                }
            } else if next.high > self.high && pre.low > self.low && next.low >= self.low {
                if allow_top_equal == Some(-1) || next.low > self.low {
                    self.fx = FxType::Bottom;
                }
            }
        } else if pre.high < self.high && next.high < self.high && pre.low < self.low && next.low < self.low {
            self.fx = FxType::Top;
        } else if pre.high > self.high && next.high > self.high && pre.low > self.low && next.low > self.low {
            self.fx = FxType::Bottom;
        }
    }
}

// Implement Index trait for array-like access
impl Index<usize> for KLine {
    type Output = KLineUnit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl IndexMut<usize> for KLine {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

impl_handle!(KLine);

impl std::fmt::Display for KLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}~{} {}->{}", self.time_begin, self.time_end, self.low, self.high)
    }
} 