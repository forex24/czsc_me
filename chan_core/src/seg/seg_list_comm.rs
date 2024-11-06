use super::seg::Seg;
use super::seg_config::SegConfig;
use crate::bi::bi::Bi;
use crate::bi::bi_list::BiList;
use crate::common::{
    chan_exception::{ChanException, ErrCode},
    enums::{BiDir, LeftSegMethod, SegType},
    handle::Handle,
};

/// 线段列表通用结构
#[derive(Debug)]
pub struct SegListComm<T> {
    pub lst: Vec<Handle<Seg<T>>>,
    pub lv: SegType,
    pub config: SegConfig,
}

impl<T> SegListComm<T>
where
    T: Clone + std::fmt::Debug,
{
    /// Create a new SegListComm instance
    pub fn new(seg_config: Option<SegConfig>, lv: Option<SegType>) -> Self {
        let config = seg_config.unwrap_or_default();
        let lv = lv.unwrap_or(SegType::Bi);

        let mut seg_list = Self {
            lst: Vec::new(),
            lv,
            config,
        };
        seg_list.do_init();
        seg_list
    }

    /// Initialize the list
    pub fn do_init(&mut self) {
        self.lst.clear();
    }

    /// Get length of the list
    pub fn len(&self) -> usize {
        self.lst.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    /// Get item by index
    pub fn get(&self, index: usize) -> Option<&Handle<Seg<T>>> {
        self.lst.get(index)
    }

    /// Get slice of items
    pub fn get_slice(&self, start: usize, end: Option<usize>) -> &[Handle<Seg<T>>] {
        let end = end.unwrap_or(self.lst.len());
        &self.lst[start..end]
    }

    /// Check if left bi breaks
    pub fn left_bi_break(&self, bi_lst: &[Handle<T>]) -> bool {
        if self.lst.is_empty() {
            return false;
        }

        let last_seg_end_bi = &self.lst.last().unwrap().end_bi;
        for bi in bi_lst.iter().skip(last_seg_end_bi.index() + 1) {
            if last_seg_end_bi.is_up() && bi.high() > last_seg_end_bi.high() {
                return true;
            } else if last_seg_end_bi.is_down() && bi.low() < last_seg_end_bi.low() {
                return true;
            }
        }
        false
    }

    /// Find peak bi
    pub fn find_peak_bi<'a, I>(bi_iter: I, is_high: bool) -> Option<Handle<T>>
    where
        I: IntoIterator<Item = &'a Handle<T>>,
    {
        let mut peak_val = if is_high {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
        let mut peak_bi = None;

        for bi in bi_iter {
            let should_update = if is_high {
                bi.borrow().get_end_val() >= peak_val && bi.borrow().is_up()
            } else {
                bi.borrow().get_end_val() <= peak_val && bi.borrow().is_down()
            };

            if should_update {
                if let (Some(pre), Some(pre_pre)) = (
                    bi.borrow().pre(),
                    bi.borrow().pre().and_then(|b| b.borrow().pre()),
                ) {
                    let should_skip = if is_high {
                        pre_pre.borrow().get_end_val() > bi.borrow().get_end_val()
                    } else {
                        pre_pre.borrow().get_end_val() < bi.borrow().get_end_val()
                    };
                    if should_skip {
                        continue;
                    }
                }
                peak_val = bi.borrow().get_end_val();
                peak_bi = Some(bi.clone());
            }
        }
        peak_bi
    }

    /// Collect first segment
    pub fn collect_first_seg(&mut self, bi_lst: &[Handle<T>]) -> Result<(), ChanException> {
        if bi_lst.len() < 3 {
            return Ok(());
        }

        match self.config.left_method {
            LeftSegMethod::Peak => {
                let high = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().high())
                    .fold(f64::NEG_INFINITY, f64::max);
                let low = bi_lst
                    .iter()
                    .map(|bi| bi.borrow().low())
                    .fold(f64::INFINITY, f64::min);

                if (high - bi_lst[0].borrow().get_begin_val()).abs()
                    >= (low - bi_lst[0].borrow().get_begin_val()).abs()
                {
                    let peak_bi =
                        Self::find_peak_bi(bi_lst.iter(), true).expect("Peak bi should exist");
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        Some(false),
                        Some(BiDir::Up),
                        Some(false),
                        Some("0seg_find_high".to_string()),
                    )?;
                } else {
                    let peak_bi =
                        Self::find_peak_bi(bi_lst.iter(), false).expect("Peak bi should exist");
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        Some(false),
                        Some(BiDir::Down),
                        Some(false),
                        Some("0seg_find_low".to_string()),
                    )?;
                }
                self.collect_left_as_seg(bi_lst)?;
            }
            LeftSegMethod::All => {
                let dir = if bi_lst.last().unwrap().get_end_val() >= bi_lst[0].get_begin_val() {
                    BiDir::Up
                } else {
                    BiDir::Down
                };
                self.add_new_seg(
                    bi_lst,
                    bi_lst.last().unwrap().index(),
                    Some(false),
                    Some(dir),
                    Some(false),
                    Some("0seg_collect_all".to_string()),
                )?;
            }
            _ => {
                return Err(ChanException::new(
                    format!("unknown seg left_method = {:?}", self.config.left_method),
                    ErrCode::ParaError,
                ))
            }
        }
        Ok(())
    }

    /// Collect left segment using peak method
    pub fn collect_left_seg_peak_method(
        &mut self,
        last_seg_end_bi: &Handle<T>,
        bi_lst: &[Handle<T>],
    ) -> Result<(), ChanException> {
        if last_seg_end_bi.borrow().is_down() {
            if let Some(peak_bi) =
                Self::find_peak_bi(bi_lst[last_seg_end_bi.borrow().idx + 3..].iter(), true)
            {
                if peak_bi.borrow().idx - last_seg_end_bi.borrow().idx >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        Some(false),
                        Some(BiDir::Up),
                        None,
                        Some("collectleft_find_high".to_string()),
                    )?;
                }
            }
        } else {
            if let Some(peak_bi) =
                Self::find_peak_bi(bi_lst[last_seg_end_bi.borrow().idx + 3..].iter(), false)
            {
                if peak_bi.borrow().idx - last_seg_end_bi.borrow().idx >= 3 {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.borrow().idx,
                        Some(false),
                        Some(BiDir::Down),
                        None,
                        Some("collectleft_find_low".to_string()),
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Try to add new segment
    pub fn try_add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: Option<bool>,
        seg_dir: Option<BiDir>,
        split_first_seg: Option<bool>,
        reason: Option<String>,
    ) -> Result<(), ChanException> {
        let split_first_seg = split_first_seg.unwrap_or(true);

        if self.lst.is_empty() && split_first_seg && end_bi_idx >= 3 {
            if let Some(peak_bi) = Self::find_peak_bi(
                &bi_lst[..=end_bi_idx - 3].iter().rev(),
                bi_lst[end_bi_idx].is_down(),
            ) {
                let should_split = if peak_bi.is_down() {
                    peak_bi.low() < bi_lst[0].low() || peak_bi.index() == 0
                } else {
                    peak_bi.high() > bi_lst[0].high() || peak_bi.index() == 0
                };

                if should_split {
                    self.add_new_seg(
                        bi_lst,
                        peak_bi.index(),
                        Some(false),
                        Some(peak_bi.dir()),
                        None,
                        Some("split_first_1st".to_string()),
                    )?;
                    self.add_new_seg(
                        bi_lst,
                        end_bi_idx,
                        Some(false),
                        None,
                        None,
                        Some("split_first_2nd".to_string()),
                    )?;
                    return Ok(());
                }
            }
        }

        let bi1_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().end_bi.index() + 1
        };
        let bi1 = &bi_lst[bi1_idx];
        let bi2 = &bi_lst[end_bi_idx];

        let new_seg = Seg::new(
            self.lst.len(),
            bi1.clone(),
            bi2.clone(),
            is_sure,
            seg_dir,
            reason,
        )?;

        if !self.lst.is_empty() {
            let last_seg = self.lst.last().unwrap();
            new_seg.borrow_mut().pre = Some(last_seg.clone());
            last_seg.borrow_mut().next = Some(new_seg.clone());
        }

        new_seg
            .borrow_mut()
            .update_bi_list(bi_lst, bi1_idx, end_bi_idx);
        self.lst.push(new_seg);

        Ok(())
    }

    /// Add new segment
    pub fn add_new_seg(
        &mut self,
        bi_lst: &[Handle<T>],
        end_bi_idx: usize,
        is_sure: Option<bool>,
        seg_dir: Option<BiDir>,
        split_first_seg: Option<bool>,
        reason: Option<String>,
    ) -> Result<bool, ChanException> {
        match self.try_add_new_seg(
            bi_lst,
            end_bi_idx,
            is_sure,
            seg_dir,
            split_first_seg,
            reason,
        ) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.errcode == ErrCode::SegEndValueErr && self.lst.is_empty() {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Check if sure segment exists
    pub fn exist_sure_seg(&self) -> bool {
        self.lst.iter().any(|seg| seg.is_sure)
    }

    /// Collect left segment as a new segment
    pub fn collect_left_as_seg(&mut self, bi_lst: &[Handle<T>]) -> Result<(), ChanException> {
        if self.lst.is_empty() {
            return Ok(());
        }

        let last_seg_end_bi = &self.lst.last().unwrap().borrow().end_bi;
        let left_bi_cnt = bi_lst.len() - last_seg_end_bi.borrow().idx - 1;

        if left_bi_cnt >= 3 {
            let last_bi = bi_lst.last().unwrap();
            let dir = if last_bi.borrow().get_end_val() >= last_seg_end_bi.borrow().get_end_val() {
                BiDir::Up
            } else {
                BiDir::Down
            };
            self.add_new_seg(
                bi_lst,
                last_bi.borrow().idx,
                Some(false),
                Some(dir),
                None,
                Some("collect_left".to_string()),
            )?;
        }
        Ok(())
    }

    /// Get iterator over segments
    pub fn iter(&self) -> impl Iterator<Item = &Handle<Seg<T>>> {
        self.lst.iter()
    }

    /// Get mutable iterator over segments
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Handle<Seg<T>>> {
        self.lst.iter_mut()
    }

    /// Get last segment
    pub fn last(&self) -> Option<&Handle<Seg<T>>> {
        self.lst.last()
    }

    /// Get first segment
    pub fn first(&self) -> Option<&Handle<Seg<T>>> {
        self.lst.first()
    }

    /// Get segment by index
    pub fn get_seg(&self, idx: usize) -> Option<&Handle<Seg<T>>> {
        self.lst.get(idx)
    }

    /// Update segment list
    pub fn update(&mut self, bi_list: &BiList<T>) -> Result<(), ChanException> {
        // Clear segments after last sure segment
        if let Some(last_sure_idx) = self.lst.iter().rposition(|seg| seg.borrow().is_sure) {
            self.lst.truncate(last_sure_idx + 1);
        } else {
            self.lst.clear();
        }

        // If empty, collect first segment
        if self.lst.is_empty() {
            self.collect_first_seg(&bi_list.lst)?;
            return Ok(());
        }

        // Process remaining segments based on left method
        let last_seg_end_bi = self.lst.last().unwrap().borrow().end_bi.clone();
        match self.config.left_method {
            LeftSegMethod::Peak => {
                self.collect_left_seg_peak_method(&last_seg_end_bi, &bi_list.lst)?;
            }
            LeftSegMethod::All => {
                if bi_list.lst.len() - last_seg_end_bi.borrow().idx > 3 {
                    let last_bi = bi_list.lst.last().unwrap();
                    let dir = if last_bi.borrow().get_end_val()
                        >= last_seg_end_bi.borrow().get_end_val()
                    {
                        BiDir::Up
                    } else {
                        BiDir::Down
                    };
                    self.add_new_seg(
                        &bi_list.lst,
                        last_bi.borrow().idx,
                        Some(false),
                        Some(dir),
                        None,
                        Some("update_collect_all".to_string()),
                    )?;
                }
            }
            _ => {
                return Err(ChanException::new(
                    format!("unknown seg left_method = {:?}", self.config.left_method),
                    ErrCode::ParaError,
                ))
            }
        }
        Ok(())
    }

    /// Get segment direction
    pub fn get_seg_dir(&self, bi_lst: &[Handle<T>], end_bi_idx: usize) -> BiDir {
        let start_idx = if self.lst.is_empty() {
            0
        } else {
            self.lst.last().unwrap().borrow().end_bi.borrow().idx + 1
        };
        if bi_lst[end_bi_idx].borrow().get_end_val() >= bi_lst[start_idx].borrow().get_begin_val() {
            BiDir::Up
        } else {
            BiDir::Down
        }
    }

    /// Check if segment list is valid
    pub fn is_valid(&self) -> bool {
        if self.lst.len() < 2 {
            return true;
        }

        for i in 1..self.lst.len() {
            let prev_seg = self.lst[i - 1].borrow();
            let curr_seg = self.lst[i].borrow();

            // Check segment continuity
            if prev_seg.end_bi.borrow().idx + 1 != curr_seg.bi_list[0].borrow().idx {
                return false;
            }

            // Check segment direction alternation
            if prev_seg.dir == curr_seg.dir {
                return false;
            }
        }
        true
    }

    /// Clear the segment list
    pub fn clear(&mut self) {
        self.lst.clear();
    }
}

impl<T> std::ops::Index<usize> for SegListComm<T>
where
    T: Clone + std::fmt::Debug,
{
    type Output = Handle<Seg<T>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl<T> std::ops::IndexMut<usize> for SegListComm<T>
where
    T: Clone + std::fmt::Debug,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
}

impl<T> Default for SegListComm<T>
where
    T: Clone + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new(None, None)
    }
}
