use std::ops::{Index, IndexMut};
use crate::common::{
    enums::{FxType, KlineDir},
    chan_exception::{ChanException, ErrCode},
    handle::Handle,
};
use crate::kline::kline::KLine;
use super::{bi::Bi, bi_config::BiConfig};

/// 笔列表管理器
#[derive(Debug)]
pub struct BiList {
    pub bi_list: Vec<Bi>,
    pub last_end: Option<KLine>,  // 最后一笔的尾部
    pub config: BiConfig,
    pub free_klc_lst: Vec<KLine>, // 仅仅用作第一笔未画出来之前的缓存，为了获得更精准的结果而已
}

impl BiList {
    /// Create a new BiList instance
    pub fn new(bi_conf: BiConfig) -> Self {
        Self {
            bi_list: Vec::new(),
            last_end: None,
            config: bi_conf,
            free_klc_lst: Vec::new(),
        }
    }

    /// Try to create the first Bi
    pub fn try_create_first_bi(&mut self, klc: KLine) -> Result<bool, ChanException> {
        for exist_free_klc in &self.free_klc_lst {
            if exist_free_klc.fx() == klc.fx() {
                continue;
            }
            if self.can_make_bi(&klc, exist_free_klc, false)? {
                self.add_new_bi(exist_free_klc.clone(), klc.clone(), true)?;
                self.last_end = Some(klc);
                return Ok(true);
            }
        }
        self.free_klc_lst.push(klc.clone());
        self.last_end = Some(klc);
        Ok(false)
    }

    /// Update Bi with new KLine
    pub fn update_bi(&mut self, klc: &KLine, last_klc: &KLine, cal_virtual: bool) -> Result<bool, ChanException> {
        let flag1 = self.update_bi_sure(klc)?;
        if cal_virtual {
            let flag2 = self.try_add_virtual_bi(last_klc, false)?;
            Ok(flag1 || flag2)
        } else {
            Ok(flag1)
        }
    }

    /// Check if can update peak
    pub fn can_update_peak(&self, klc: &KLine) -> Result<bool, ChanException> {
        if self.config.bi_allow_sub_peak || self.bi_list.len() < 2 {
            return Ok(false);
        }
        
        let last_bi = self.bi_list.last().unwrap();
        let second_last_bi = &self.bi_list[self.bi_list.len() - 2];
        
        if last_bi.is_down() && klc.high() < last_bi.get_begin_val() {
            return Ok(false);
        }
        if last_bi.is_up() && klc.low() > last_bi.get_begin_val() {
            return Ok(false);
        }
        if !end_is_peak(&second_last_bi.begin_klc(), klc)? {
            return Ok(false);
        }
        if last_bi.is_down() && last_bi.get_end_val() < second_last_bi.get_begin_val() {
            return Ok(false);
        }
        if last_bi.is_up() && last_bi.get_end_val() > second_last_bi.get_begin_val() {
            return Ok(false);
        }
        Ok(true)
    }

    /// Update peak with new KLine
    pub fn update_peak(&mut self, klc: &KLine, for_virtual: bool) -> Result<bool, ChanException> {
        if !self.can_update_peak(klc)? {
            return Ok(false);
        }
        
        let tmp_last_bi = self.bi_list.pop().unwrap();
        if !self.try_update_end(klc, for_virtual)? {
            self.bi_list.push(tmp_last_bi);
            Ok(false)
        } else {
            if for_virtual {
                self.bi_list.last_mut().unwrap().append_sure_end(tmp_last_bi.end_klc().clone());
            }
            Ok(true)
        }
    }

    /// Update confirmed Bi
    pub fn update_bi_sure(&mut self, klc: &KLine) -> Result<bool, ChanException> {
        let tmp_end = self.get_last_klu_of_last_bi();
        self.delete_virtual_bi()?;
        
        if klc.fx() == FxType::Unknown {
            return Ok(tmp_end != self.get_last_klu_of_last_bi());
        }
        
        if self.last_end.is_none() || self.bi_list.is_empty() {
            return self.try_create_first_bi(klc.clone());
        }
        
        let last_end = self.last_end.as_ref().unwrap();
        if klc.fx() == last_end.fx() {
            Ok(self.try_update_end(klc, false)?)
        } else if self.can_make_bi(klc, last_end, false)? {
            self.add_new_bi(last_end.clone(), klc.clone(), true)?;
            self.last_end = Some(klc.clone());
            Ok(true)
        } else if self.update_peak(klc, false)? {
            Ok(true)
        } else {
            Ok(tmp_end != self.get_last_klu_of_last_bi())
        }
    }

    /// Delete virtual Bi
    pub fn delete_virtual_bi(&mut self) -> Result<(), ChanException> {
        if !self.bi_list.is_empty() && !self.bi_list.last().unwrap().is_sure() {
            let sure_end_list: Vec<_> = self.bi_list.last().unwrap().sure_end().to_vec();
            if !sure_end_list.is_empty() {
                let last_bi = self.bi_list.last_mut().unwrap();
                last_bi.restore_from_virtual_end(&sure_end_list[0])?;
                self.last_end = Some(last_bi.end_klc().clone());
                
                for sure_end in &sure_end_list[1..] {
                    self.add_new_bi(self.last_end.as_ref().unwrap().clone(), sure_end.clone(), true)?;
                    self.last_end = Some(self.bi_list.last().unwrap().end_klc().clone());
                }
            } else {
                self.bi_list.pop();
            }
        }
        
        self.last_end = if !self.bi_list.is_empty() {
            Some(self.bi_list.last().unwrap().end_klc().clone())
        } else {
            None
        };
        
        if !self.bi_list.is_empty() {
            self.bi_list.last_mut().unwrap().set_next(None);
        }
        
        Ok(())
    }

    /// Try to add virtual Bi
    pub fn try_add_virtual_bi(&mut self, klc: &KLine, need_del_end: bool) -> Result<bool, ChanException> {
        if need_del_end {
            self.delete_virtual_bi()?;
        }
        
        if self.bi_list.is_empty() {
            return Ok(false);
        }
        
        let last_bi = self.bi_list.last().unwrap();
        if klc.idx() == last_bi.end_klc().idx() {
            return Ok(false);
        }
        
        if (last_bi.is_up() && klc.high() >= last_bi.end_klc().high()) || 
           (last_bi.is_down() && klc.low() <= last_bi.end_klc().low()) {
            self.bi_list.last_mut().unwrap().update_virtual_end(klc)?;
            return Ok(true);
        }
        
        let mut tmp_klc = Some(klc.clone());
        while let Some(current_klc) = tmp_klc {
            if current_klc.idx() <= last_bi.end_klc().idx() {
                break;
            }
            
            if self.can_make_bi(&current_klc, last_bi.end_klc(), true)? {
                self.add_new_bi(self.last_end.as_ref().unwrap().clone(), current_klc, false)?;
                return Ok(true);
            } else if self.update_peak(&current_klc, true)? {
                return Ok(true);
            }
            
            tmp_klc = current_klc.prev();
        }
        
        Ok(false)
    }

    /// Add new Bi to the list
    pub fn add_new_bi(&mut self, pre_klc: KLine, cur_klc: KLine, is_sure: bool) -> Result<(), ChanException> {
        let new_bi = Bi::new(pre_klc, cur_klc, self.bi_list.len(), is_sure, &Box::new(self.bi_list.clone()))?;
        
        if !self.bi_list.is_empty() {
            let last_bi = self.bi_list.last_mut().unwrap();
            last_bi.set_next(Some(&new_bi));
            new_bi.set_pre(Some(last_bi));
        }
        
        self.bi_list.push(new_bi);
        Ok(())
    }

    /// Check if satisfies Bi span requirements
    pub fn satisfy_bi_span(&self, klc: &KLine, last_end: &KLine) -> Result<bool, ChanException> {
        let bi_span = self.get_klc_span(klc, last_end)?;
        
        if self.config.is_strict {
            return Ok(bi_span >= 4);
        }
        
        let mut unit_kl_cnt = 0;
        let mut tmp_klc = last_end.next();
        
        while let Some(current_klc) = tmp_klc {
            unit_kl_cnt += current_klc.lst.len();
            
            if current_klc.next().is_none() {
                return Ok(false);
            }
            
            if current_klc.next().unwrap().idx() < klc.idx() {
                tmp_klc = current_klc.next();
            } else {
                break;
            }
        }
        
        Ok(bi_span >= 3 && unit_kl_cnt >= 3)
    }

    /// Get KLine span
    pub fn get_klc_span(&self, klc: &KLine, last_end: &KLine) -> Result<usize, ChanException> {
        let mut span = klc.idx() - last_end.idx();
        
        if !self.config.gap_as_kl {
            return Ok(span);
        }
        
        if span >= 4 {
            return Ok(span);
        }
        
        let mut tmp_klc = Some(last_end.clone());
        while let Some(current_klc) = tmp_klc {
            if current_klc.idx() >= klc.idx() {
                break;
            }
            
            if current_klc.has_gap_with_next() {
                span += 1;
            }
            
            tmp_klc = current_klc.next();
        }
        
        Ok(span)
    }

    /// Check if can make Bi
    pub fn can_make_bi(&self, klc: &KLine, last_end: &KLine, for_virtual: bool) -> Result<bool, ChanException> {
        let satisfy_span = if self.config.bi_algo == "fx" {
            true
        } else {
            self.satisfy_bi_span(klc, last_end)?
        };
        
        if !satisfy_span {
            return Ok(false);
        }
        
        if !last_end.check_fx_valid(klc, &self.config.bi_fx_check, for_virtual)? {
            return Ok(false);
        }
        
        if self.config.bi_end_is_peak && !end_is_peak(last_end, klc)? {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Try to update end
    pub fn try_update_end(&mut self, klc: &KLine, for_virtual: bool) -> Result<bool, ChanException> {
        if self.bi_list.is_empty() {
            return Ok(false);
        }
        
        let last_bi = self.bi_list.last_mut().unwrap();
        let check_top = if for_virtual {
            klc.dir() == KlineDir::Up
        } else {
            klc.fx() == FxType::Top
        };
        
        let check_bottom = if for_virtual {
            klc.dir() == KlineDir::Down
        } else {
            klc.fx() == FxType::Bottom
        };
        
        if (last_bi.is_up() && check_top && klc.high() >= last_bi.get_end_val()) ||
           (last_bi.is_down() && check_bottom && klc.low() <= last_bi.get_end_val()) {
            if for_virtual {
                last_bi.update_virtual_end(klc)?;
            } else {
                last_bi.update_new_end(klc)?;
            }
            self.last_end = Some(klc.clone());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get last KLineUnit of last Bi
    pub fn get_last_klu_of_last_bi(&self) -> Option<usize> {
        self.bi_list.last().map(|bi| bi.get_end_klu().unwrap().idx())
    }
}

/// Check if end is peak
pub fn end_is_peak(last_end: &KLine, cur_end: &KLine) -> Result<bool, ChanException> {
    match last_end.fx() {
        FxType::Bottom => {
            let cmp_thred = cur_end.high();
            let mut klc = last_end.next();
            
            while let Some(current_klc) = klc {
                if current_klc.idx() >= cur_end.idx() {
                    return Ok(true);
                }
                if current_klc.high() > cmp_thred {
                    return Ok(false);
                }
                klc = current_klc.next();
            }
            Ok(true)
        },
        FxType::Top => {
            let cmp_thred = cur_end.low();
            let mut klc = last_end.next();
            
            while let Some(current_klc) = klc {
                if current_klc.idx() >= cur_end.idx() {
                    return Ok(true);
                }
                if current_klc.low() < cmp_thred {
                    return Ok(false);
                }
                klc = current_klc.next();
            }
            Ok(true)
        },
        _ => Ok(true),
    }
}

// Implement Index trait for array-like access
impl Index<usize> for BiList {
    type Output = Bi;

    fn index(&self, index: usize) -> &Self::Output {
        &self.bi_list[index]
    }
}

impl IndexMut<usize> for BiList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.bi_list[index]
    }
}

impl std::fmt::Display for BiList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bi in &self.bi_list {
            writeln!(f, "{}", bi)?;
        }
        Ok(())
    }
} 