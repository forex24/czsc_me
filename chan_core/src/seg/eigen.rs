use std::ops::{Index, IndexMut};
use crate::common::{
    enums::{BiDir, FxType, KlineDir},
    chan_exception::{ChanException, ErrCode},
    time::Time,
};
use crate::bi::bi::Bi;
use crate::kline::kline_unit::KLineUnit;
use crate::seg::seg::Seg;

/// 组合项目特征
pub trait CombineItemTrait {
    fn time_begin(&self) -> Time;
    fn time_end(&self) -> Time;
    fn high(&self) -> f64;
    fn low(&self) -> f64;
    fn as_kline_unit(&self) -> Option<&KLineUnit> { None }
    fn dir(&self) -> BiDir;
    fn idx(&self) -> usize;
}

impl CombineItemTrait for Bi {
    fn time_begin(&self) -> Time { self.begin_klc().idx() }
    fn time_end(&self) -> Time { self.end_klc().idx() }
    fn high(&self) -> f64 { self.high() }
    fn low(&self) -> f64 { self.low() }
    fn dir(&self) -> BiDir { self.dir() }
    fn idx(&self) -> usize { self.idx() }
}

impl CombineItemTrait for KLineUnit {
    fn time_begin(&self) -> Time { self.time() }
    fn time_end(&self) -> Time { self.time() }
    fn high(&self) -> f64 { self.high }
    fn low(&self) -> f64 { self.low }
    fn as_kline_unit(&self) -> Option<&KLineUnit> { Some(self) }
    fn dir(&self) -> BiDir { self.dir.into() }
    fn idx(&self) -> usize { self.idx }
}

impl CombineItemTrait for Seg<Bi> {
    fn time_begin(&self) -> Time { self.start_bi().begin_klc().idx() }
    fn time_end(&self) -> Time { self.end_bi().end_klc().idx() }
    fn high(&self) -> f64 { self.high() }
    fn low(&self) -> f64 { self.low() }
    fn dir(&self) -> BiDir { self.dir() }
    fn idx(&self) -> usize { self.idx() }
}

/// 特征序列结构，用于线段的生成
#[derive(Debug)]
pub struct Eigen<T: CombineItemTrait> {
    pub time_begin: Time,
    pub time_end: Time,
    pub high: f64,
    pub low: f64,
    pub lst: Vec<T>,
    pub dir: BiDir,
    pub fx: FxType,
    pub pre: Option<Box<Eigen<T>>>,
    pub next: Option<Box<Eigen<T>>>,
    pub gap: bool,
}

impl<T: CombineItemTrait> Eigen<T> {
    /// Create a new Eigen instance
    pub fn new(kl_unit: T, dir: BiDir) -> Result<Self, ChanException> {
        Ok(Self {
            time_begin: kl_unit.time_begin(),
            time_end: kl_unit.time_end(),
            high: kl_unit.high(),
            low: kl_unit.low(),
            lst: vec![kl_unit],
            dir,
            fx: FxType::Unknown,
            pre: None,
            next: None,
            gap: false,
        })
    }

    /// Clean cache (placeholder for future cache implementation)
    pub fn clean_cache(&mut self) {
        // TODO: Implement caching mechanism if needed
    }

    /// Test if can combine with another item
    pub fn test_combine(&self, item: &T, exclude_included: bool, allow_top_equal: Option<i32>) -> Result<KlineDir, ChanException> {
        if self.high >= item.high() && self.low <= item.low() {
            return Ok(KlineDir::Combine);
        }
        if self.high <= item.high() && self.low >= item.low() {
            match allow_top_equal {
                Some(1) if self.high == item.high() && self.low > item.low() => return Ok(KlineDir::Down),
                Some(-1) if self.low == item.low() && self.high < item.high() => return Ok(KlineDir::Up),
                _ => return Ok(if exclude_included { KlineDir::Included } else { KlineDir::Combine }),
            }
        }
        if self.high > item.high() && self.low > item.low() {
            return Ok(KlineDir::Down);
        }
        if self.high < item.high() && self.low < item.low() {
            return Ok(KlineDir::Up);
        }
        
        Err(ChanException::new(
            "combine type unknown".into(),
            ErrCode::CombinerErr,
        ))
    }

    /// Try to add a new unit
    pub fn try_add(&mut self, unit_kl: T, exclude_included: bool, allow_top_equal: Option<i32>) -> Result<KlineDir, ChanException> {
        let dir = self.test_combine(&unit_kl, exclude_included, allow_top_equal)?;
        
        if dir == KlineDir::Combine {
            if let Some(klu) = unit_kl.as_kline_unit() {
                // Handle KLineUnit specific logic
                klu.set_klc(self)?;
            }
            
            match self.dir {
                BiDir::Up => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.high() != self.high {
                        self.high = self.high.max(unit_kl.high());
                        self.low = self.low.max(unit_kl.low());
                    }
                },
                BiDir::Down => {
                    if unit_kl.high() != unit_kl.low() || unit_kl.low() != self.low {
                        self.high = self.high.min(unit_kl.high());
                        self.low = self.low.min(unit_kl.low());
                    }
                },
            }
            
            self.lst.push(unit_kl);
            self.time_end = self.lst.last().unwrap().time_end();
            self.clean_cache();
        }
        
        Ok(dir)
    }

    /// Get peak item
    pub fn get_peak_klu(&self, is_high: bool) -> Result<&T, ChanException> {
        if is_high {
            self.get_high_peak_klu()
        } else {
            self.get_low_peak_klu()
        }
    }

    /// Get high peak item
    pub fn get_high_peak_klu(&self) -> Result<&T, ChanException> {
        for kl in self.lst.iter().rev() {
            if kl.high() == self.high {
                return Ok(kl);
            }
        }
        Err(ChanException::new(
            "can't find peak...".into(),
            ErrCode::CombinerErr,
        ))
    }

    /// Get low peak item
    pub fn get_low_peak_klu(&self) -> Result<&T, ChanException> {
        for kl in self.lst.iter().rev() {
            if kl.low() == self.low {
                return Ok(kl);
            }
        }
        Err(ChanException::new(
            "can't find peak...".into(),
            ErrCode::CombinerErr,
        ))
    }

    /// Update fractal type
    pub fn update_fx(
        &mut self,
        pre: &Self,
        next: &Self,
        exclude_included: bool,
        allow_top_equal: Option<i32>,
    ) -> Result<(), ChanException> {
        self.set_next(Some(Box::new(next.clone())));
        self.set_pre(Some(Box::new(pre.clone())));
        next.set_pre(Some(Box::new(self.clone())));

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

        // Check for gaps
        self.gap = match self.fx {
            FxType::Top if pre.high < self.low => true,
            FxType::Bottom if pre.low > self.high => true,
            _ => false,
        };

        self.clean_cache();
        Ok(())
    }

    // Setters
    pub fn set_pre(&mut self, pre: Option<Box<Self>>) {
        self.pre = pre;
        self.clean_cache();
    }

    pub fn set_next(&mut self, next: Option<Box<Self>>) {
        self.next = next;
        self.clean_cache();
    }

    /// Get peak Bi index
    pub fn get_peak_bi_idx(&self) -> Result<usize, ChanException> {
        assert!(self.fx != FxType::Unknown, "Fractal type must be known");
        
        let bi_dir = self.lst[0].dir();
        
        Ok(match bi_dir {
            BiDir::Up => {
                // 下降线段
                self.get_peak_klu(false)?.idx() - 1
            },
            BiDir::Down => {
                self.get_peak_klu(true)?.idx() - 1
            },
        })
    }
}

// Implement standard traits
impl<T: CombineItemTrait> Index<usize> for Eigen<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T: CombineItemTrait> IndexMut<usize> for Eigen<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

impl<T: CombineItemTrait> std::fmt::Display for Eigen<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "{}~{} gap={} fx={}", 
            self.lst[0].idx(),
            self.lst.last().unwrap().idx(),
            self.gap,
            self.fx
        )
    }
} 