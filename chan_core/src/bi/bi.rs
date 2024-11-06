use crate::common::{
    enums::{BiDir, BiType, DataField, FxType, MacdAlgo},
    chan_exception::{ChanException, ErrCode},
    handle::Handle,
};
use crate::kline::{kline::KLine, kline_unit::KLineUnit};
use crate::seg::seg::Seg;
use crate::bs_point::bs_point::BSPoint;
use crate::impl_handle;

/// 笔结构，表示一段方向明确的走势
#[derive(Debug)]
pub struct Bi {
    pub handle: Handle<Self>,
    begin_klc: KLine,
    end_klc: KLine,
    dir: BiDir,
    idx: usize,
    bi_type: BiType,
    is_sure: bool,
    sure_end: Vec<KLine>,
    seg_idx: Option<usize>,
    pub parent_seg: Option<Seg<Self>>,  // 在哪个线段里面
    pub bsp: Option<BSPoint>,           // 尾部是不是买卖点
    
    // 缓存相关字段
    cached_begin_val: Option<f64>,
    cached_end_val: Option<f64>,
    cached_begin_klu: Option<KLineUnit>,
    cached_end_klu: Option<KLineUnit>,
    cached_amp: Option<f64>,
    cached_klu_cnt: Option<usize>,
    cached_klc_cnt: Option<usize>,
    cached_high: Option<f64>,
    cached_low: Option<f64>,
}

impl Bi {
    /// Create a new Bi instance
    pub fn new(begin_klc: KLine, end_klc: KLine, idx: usize, is_sure: bool, box_vec: &Box<Vec<Self>>) -> Result<Self, ChanException> {
        let mut bi = Self {
            handle: Handle::new(box_vec, idx),
            begin_klc,
            end_klc,
            dir: BiDir::Up, // Will be set in set()
            idx,
            bi_type: BiType::Strict,
            is_sure,
            sure_end: Vec::new(),
            seg_idx: None,
            parent_seg: None,
            bsp: None,
            
            // Initialize cache fields
            cached_begin_val: None,
            cached_end_val: None,
            cached_begin_klu: None,
            cached_end_klu: None,
            cached_amp: None,
            cached_klu_cnt: None,
            cached_klc_cnt: None,
            cached_high: None,
            cached_low: None,
        };
        
        bi.set(begin_klc, end_klc)?;
        Ok(bi)
    }

    /// Clean all cached values
    pub fn clean_cache(&mut self) {
        self.cached_begin_val = None;
        self.cached_end_val = None;
        self.cached_begin_klu = None;
        self.cached_end_klu = None;
        self.cached_amp = None;
        self.cached_klu_cnt = None;
        self.cached_klc_cnt = None;
        self.cached_high = None;
        self.cached_low = None;
    }

    // Getters
    pub fn begin_klc(&self) -> &KLine { &self.begin_klc }
    pub fn end_klc(&self) -> &KLine { &self.end_klc }
    pub fn dir(&self) -> BiDir { self.dir }
    pub fn idx(&self) -> usize { self.idx }
    pub fn bi_type(&self) -> BiType { self.bi_type }
    pub fn is_sure(&self) -> bool { self.is_sure }
    pub fn sure_end(&self) -> &[KLine] { &self.sure_end }
    pub fn seg_idx(&self) -> Option<usize> { self.seg_idx }

    /// Set segment index
    pub fn set_seg_idx(&mut self, idx: usize) {
        self.seg_idx = Some(idx);
    }

    /// Check if the Bi is valid
    fn check(&self) -> Result<(), ChanException> {
        if self.is_down() {
            if self.begin_klc.high() <= self.end_klc.low() {
                return Err(ChanException::new(
                    format!("{}:{}~{}笔的方向和收尾位置不一致!", 
                        self.idx,
                        self.begin_klc[0].time,
                        self.end_klc.last().unwrap().time
                    ),
                    ErrCode::BiErr
                ));
            }
        } else if self.begin_klc.low() >= self.end_klc.high() {
            return Err(ChanException::new(
                format!("{}:{}~{}笔的方向和收尾位置不一致!", 
                    self.idx,
                    self.begin_klc[0].time,
                    self.end_klc.last().unwrap().time
                ),
                ErrCode::BiErr
            ));
        }
        Ok(())
    }

    /// Set begin and end KLines
    pub fn set(&mut self, begin_klc: KLine, end_klc: KLine) -> Result<(), ChanException> {
        self.begin_klc = begin_klc;
        self.end_klc = end_klc;
        
        self.dir = match self.begin_klc.fx() {
            FxType::Bottom => BiDir::Up,
            FxType::Top => BiDir::Down,
            _ => return Err(ChanException::new(
                "ERROR DIRECTION when creating bi".into(),
                ErrCode::BiErr
            )),
        };
        
        self.check()?;
        self.clean_cache();
        Ok(())
    }

    // Helper methods
    pub fn is_up(&self) -> bool { self.dir == BiDir::Up }
    pub fn is_down(&self) -> bool { self.dir == BiDir::Down }

    /// Get begin value with caching
    pub fn get_begin_val(&mut self) -> f64 {
        if let Some(val) = self.cached_begin_val {
            return val;
        }
        let val = if self.is_up() {
            self.begin_klc.low()
        } else {
            self.begin_klc.high()
        };
        self.cached_begin_val = Some(val);
        val
    }

    /// Get end value with caching
    pub fn get_end_val(&mut self) -> f64 {
        if let Some(val) = self.cached_end_val {
            return val;
        }
        let val = if self.is_up() {
            self.end_klc.high()
        } else {
            self.end_klc.low()
        };
        self.cached_end_val = Some(val);
        val
    }

    /// Get begin KLineUnit with caching
    pub fn get_begin_klu(&mut self) -> Result<KLineUnit, ChanException> {
        if let Some(ref klu) = self.cached_begin_klu {
            return Ok(klu.clone());
        }
        let klu = if self.is_up() {
            self.begin_klc.get_peak_klu(false)?
        } else {
            self.begin_klc.get_peak_klu(true)?
        };
        self.cached_begin_klu = Some(klu.clone());
        Ok(klu)
    }

    /// Get end KLineUnit with caching
    pub fn get_end_klu(&mut self) -> Result<KLineUnit, ChanException> {
        if let Some(ref klu) = self.cached_end_klu {
            return Ok(klu.clone());
        }
        let klu = if self.is_up() {
            self.end_klc.get_peak_klu(true)?
        } else {
            self.end_klc.get_peak_klu(false)?
        };
        self.cached_end_klu = Some(klu.clone());
        Ok(klu)
    }

    /// Calculate amplitude with caching
    pub fn amp(&mut self) -> f64 {
        if let Some(amp) = self.cached_amp {
            return amp;
        }
        let amp = (self.get_end_val() - self.get_begin_val()).abs();
        self.cached_amp = Some(amp);
        amp
    }

    /// Get KLineUnit count with caching
    pub fn get_klu_cnt(&mut self) -> Result<usize, ChanException> {
        if let Some(cnt) = self.cached_klu_cnt {
            return Ok(cnt);
        }
        let cnt = self.get_end_klu()?.idx - self.get_begin_klu()?.idx + 1;
        self.cached_klu_cnt = Some(cnt);
        Ok(cnt)
    }

    /// Calculate MACD metric based on algorithm
    pub fn cal_macd_metric(&self, macd_algo: MacdAlgo, is_reverse: bool) -> Result<f64, ChanException> {
        match macd_algo {
            MacdAlgo::Area => self.cal_macd_half(is_reverse),
            MacdAlgo::Peak => self.cal_macd_peak(),
            MacdAlgo::FullArea => self.cal_macd_area(),
            MacdAlgo::Diff => self.cal_macd_diff(),
            MacdAlgo::Slope => self.cal_macd_slope(),
            MacdAlgo::Amp => self.cal_macd_amp(),
            MacdAlgo::Amount => self.cal_macd_trade_metric(DataField::FieldTurnover, false),
            MacdAlgo::Volume => self.cal_macd_trade_metric(DataField::FieldVolume, false),
            MacdAlgo::VolumeAvg => self.cal_macd_trade_metric(DataField::FieldVolume, true),
            MacdAlgo::AmountAvg => self.cal_macd_trade_metric(DataField::FieldTurnover, true),
            MacdAlgo::TurnrateAvg => self.cal_macd_trade_metric(DataField::FieldTurnrate, true),
            MacdAlgo::Rsi => self.cal_rsi(),
            _ => Err(ChanException::new(
                format!("unsupport macd_algo={:?}, should be one of area/full_area/peak/diff/slope/amp", macd_algo),
                ErrCode::ParaError,
            )),
        }
    }

    /// Calculate RSI
    fn cal_rsi(&self) -> Result<f64, ChanException> {
        let mut rsi_lst = Vec::new();
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                rsi_lst.push(klu.rsi);
            }
        }
        Ok(if self.is_down() {
            10000.0 / (rsi_lst.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + 1e-7)
        } else {
            *rsi_lst.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        })
    }

    /// Calculate MACD area
    fn cal_macd_area(&self) -> Result<f64, ChanException> {
        let mut s = 1e-7;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                s += klu.macd.macd.abs();
            }
        }
        Ok(s)
    }

    /// Calculate MACD peak
    fn cal_macd_peak(&self) -> Result<f64, ChanException> {
        let mut peak = 1e-7;
        for klc in self.klc_lst() {
            for klu in klc.lst.iter() {
                if klu.macd.macd.abs() > peak {
                    peak = klu.macd.macd.abs();
                }
            }
        }
        Ok(peak)
    }
}

impl_handle!(Bi);

impl std::fmt::Display for Bi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{} ~ {}", self.dir, self.begin_klc, self.end_klc)
    }
} 