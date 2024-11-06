use std::marker::PhantomData;
use crate::common::{
    enums::{BiDir, MacdAlgo, TrendLineSide},
    chan_exception::{ChanException, ErrCode},
    handle::Handle,
};
use crate::bi::bi::Bi;
use crate::kline::kline_unit::KLineUnit;
use crate::math::trend_line::TrendLine;
use crate::zs::zs::Zs;
use crate::buy_sell_point::bs_point::BsPoint;
use super::eigen_fx::EigenFX;

/// 线段结构
#[derive(Debug)]
pub struct Seg<T> {
    pub handle: Handle<Seg<T>>,
    pub start_bi: Handle<T>,
    pub end_bi: Handle<T>,
    pub is_sure: bool,
    pub dir: BiDir,
    pub zs_lst: Vec<Handle<Zs<T>>>,
    pub eigen_fx: Option<EigenFX<T>>,
    pub seg_idx: Option<usize>,
    pub parent_seg: Option<Handle<Seg<T>>>,
    pub pre: Option<Handle<Seg<T>>>,
    pub next: Option<Handle<Seg<T>>>,
    pub bsp: Option<Handle<BsPoint>>,
    pub bi_list: Vec<Handle<T>>,
    pub reason: String,
    pub support_trend_line: Option<TrendLine<T>>,
    pub resistance_trend_line: Option<TrendLine<T>>,
    pub ele_inside_is_sure: bool,
    _phantom: PhantomData<T>,
}

impl<T> Seg<T> 
where T: Clone + std::fmt::Debug
{
    /// Create a new Seg instance with handle
    pub fn new_with_handle(
        handle: Handle<Seg<T>>,
        start_bi: Handle<T>,
        end_bi: Handle<T>,
        is_sure: Option<bool>,
        seg_dir: Option<BiDir>,
        reason: Option<String>,
    ) -> Result<Self, ChanException> {
        let is_sure = is_sure.unwrap_or(true);
        
        assert!(
            start_bi.index() == 0 || start_bi.dir() == end_bi.dir() || !is_sure,
            "Start and end bi direction mismatch"
        );

        let dir = seg_dir.unwrap_or_else(|| end_bi.dir());
        let reason = reason.unwrap_or_else(|| "normal".to_string());
        
        let mut seg = Self {
            handle,
            start_bi,
            end_bi,
            is_sure,
            dir,
            zs_lst: Vec::new(),
            eigen_fx: None,
            seg_idx: None,
            parent_seg: None,
            pre: None,
            next: None,
            bsp: None,
            bi_list: Vec::new(),
            reason,
            support_trend_line: None,
            resistance_trend_line: None,
            ele_inside_is_sure: false,
            _phantom: PhantomData,
        };

        if end_bi.index() - start_bi.index() < 2 {
            seg.is_sure = false;
        }
        
        seg.check()?;
        Ok(seg)
    }

    /// Get index
    pub fn index(&self) -> usize {
        self.handle.index()
    }

    /// Check segment validity
    pub fn check(&self) -> Result<(), ChanException> {
        if !self.is_sure {
            return Ok(());
        }

        if self.is_down() {
            if self.start_bi.get_begin_val() < self.end_bi.get_end_val() {
                return Err(ChanException::new(
                    format!("下降线段起始点应该高于结束点! idx={}", self.index()),
                    ErrCode::SegEndValueErr,
                ));
            }
        } else if self.start_bi.get_begin_val() > self.end_bi.get_end_val() {
            return Err(ChanException::new(
                format!("上升线段起始点应该低于结束点! idx={}", self.index()),
                ErrCode::SegEndValueErr,
            ));
        }

        if self.end_bi.index() - self.start_bi.index() < 2 {
            return Err(ChanException::new(
                format!("线段({}-{})长度不能小于2! idx={}", 
                    self.start_bi.index(), self.end_bi.index(), self.index()),
                ErrCode::SegLenErr,
            ));
        }

        Ok(())
    }

    /// Add a new ZS
    pub fn add_zs(&mut self, zs: Handle<Zs<T>>) {
        self.zs_lst.insert(0, zs); // 因为中枢是反序加入的
    }

    /// Calculate KLineUnit slope
    pub fn cal_klu_slope(&self) -> f64 {
        assert!(self.end_bi.index() >= self.start_bi.index());
        (self.get_end_val() - self.get_begin_val()) / 
        (self.get_end_klu().index() as f64 - self.get_begin_klu().index() as f64) / 
        self.get_begin_val()
    }

    /// Calculate amplitude
    pub fn cal_amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()) / self.get_begin_val()
    }

    /// Calculate bi count
    pub fn cal_bi_cnt(&self) -> usize {
        self.end_bi.index() - self.start_bi.index() + 1
    }

    /// Clear ZS list
    pub fn clear_zs_lst(&mut self) {
        self.zs_lst.clear();
    }

    /// Get low value
    pub fn low(&self) -> f64 {
        if self.is_down() {
            self.end_bi.get_end_klu().low()
        } else {
            self.start_bi.get_begin_klu().low()
        }
    }

    /// Get high value
    pub fn high(&self) -> f64 {
        if self.is_up() {
            self.end_bi.get_end_klu().high()
        } else {
            self.start_bi.get_begin_klu().high()
        }
    }

    /// Check if direction is down
    pub fn is_down(&self) -> bool {
        self.dir == BiDir::Down
    }

    /// Check if direction is up
    pub fn is_up(&self) -> bool {
        self.dir == BiDir::Up
    }

    /// Get end value
    pub fn get_end_val(&self) -> f64 {
        self.end_bi.get_end_val()
    }

    /// Get begin value
    pub fn get_begin_val(&self) -> f64 {
        self.start_bi.get_begin_val()
    }

    /// Get amplitude
    pub fn amp(&self) -> f64 {
        (self.get_end_val() - self.get_begin_val()).abs()
    }

    /// Get end KLineUnit
    pub fn get_end_klu(&self) -> Handle<KLineUnit> {
        self.end_bi.get_end_klu()
    }

    /// Get begin KLineUnit
    pub fn get_begin_klu(&self) -> Handle<KLineUnit> {
        self.start_bi.get_begin_klu()
    }

    /// Get KLineUnit count
    pub fn get_klu_cnt(&self) -> usize {
        self.get_end_klu().index() - self.get_begin_klu().index() + 1
    }

    /// Calculate MACD metric
    pub fn cal_macd_metric(&self, macd_algo: MacdAlgo, is_reverse: bool) -> Result<f64, ChanException> {
        match macd_algo {
            MacdAlgo::Slope => Ok(self.cal_macd_slope()),
            MacdAlgo::Amp => Ok(self.cal_macd_amp()),
            _ => Err(ChanException::new(
                format!("unsupport macd_algo={:?} of Seg, should be one of slope/amp", macd_algo),
                ErrCode::ParaError,
            )),
        }
    }

    /// Calculate MACD slope
    pub fn cal_macd_slope(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        
        if self.is_up() {
            (end_klu.high() - begin_klu.low()) / end_klu.high() / 
            (end_klu.index() as f64 - begin_klu.index() as f64 + 1.0)
        } else {
            (begin_klu.high() - end_klu.low()) / begin_klu.high() / 
            (end_klu.index() as f64 - begin_klu.index() as f64 + 1.0)
        }
    }

    /// Calculate MACD amplitude
    pub fn cal_macd_amp(&self) -> f64 {
        let begin_klu = self.get_begin_klu();
        let end_klu = self.get_end_klu();
        
        if self.is_down() {
            (begin_klu.high() - end_klu.low()) / begin_klu.high()
        } else {
            (end_klu.high() - begin_klu.low()) / begin_klu.low()
        }
    }

    /// Update bi list
    pub fn update_bi_list(&mut self, bi_lst: &[Handle<T>], idx1: usize, idx2: usize) {
        for bi_idx in idx1..=idx2 {
            if let Some(bi) = bi_lst.get(bi_idx) {
                bi.set_parent_seg(self.handle.clone());
                self.bi_list.push(bi.clone());
            }
        }

        if self.bi_list.len() >= 3 {
            self.support_trend_line = Some(TrendLine::new(
                &self.bi_list,
                TrendLineSide::Inside,
            ));
            self.resistance_trend_line = Some(TrendLine::new(
                &self.bi_list,
                TrendLineSide::Outside,
            ));
        }
    }

    /// Get first multi bi ZS
    pub fn get_first_multi_bi_zs(&self) -> Option<Handle<Zs<T>>> {
        self.zs_lst.iter()
            .find(|zs| !zs.is_one_bi_zs())
            .cloned()
    }

    /// Get final multi bi ZS
    pub fn get_final_multi_bi_zs(&self) -> Option<Handle<Zs<T>>> {
        self.zs_lst.iter()
            .rev()
            .find(|zs| !zs.is_one_bi_zs())
            .cloned()
    }

    /// Get multi bi ZS count
    pub fn get_multi_bi_zs_cnt(&self) -> usize {
        self.zs_lst.iter()
            .filter(|zs| !zs.is_one_bi_zs())
            .count()
    }
}

impl<T> std::fmt::Display for Seg<T>
where T: Clone + std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}->{}: {:?}  {}", 
            self.start_bi.index(),
            self.end_bi.index(),
            self.dir,
            self.is_sure
        )
    }
} 