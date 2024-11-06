use crate::common::{
    enums::{BiDir, FxType, KlineDir, SegType},
    chan_exception::{ChanException, ErrCode},
    func_util::revert_bi_dir,
    handle::Handle,
};
use crate::bi::bi::Bi;
use super::eigen::Eigen;

/// 特征序列分型结构
#[derive(Debug)]
pub struct EigenFX<T> {
    pub lv: SegType,
    pub dir: BiDir,                // 线段方向
    pub ele: [Option<Eigen<T>>; 3], // 特征序列元素
    pub lst: Vec<Handle<T>>,      // 笔列表
    pub exclude_included: bool,    // 是否排除包含关系
    pub kl_dir: KlineDir,         // K线方向
    pub last_evidence_bi: Option<Handle<T>>, // 最后一个证据笔
}

impl<T> EigenFX<T> 
where T: Clone + std::fmt::Debug
{
    /// Create a new EigenFX instance
    pub fn new(dir: BiDir, exclude_included: bool, lv: SegType) -> Self {
        let kl_dir = if dir == BiDir::Up { KlineDir::Up } else { KlineDir::Down };
        
        Self {
            lv,
            dir,
            ele: [None, None, None],
            lst: Vec::new(),
            exclude_included,
            kl_dir,
            last_evidence_bi: None,
        }
    }

    /// Handle first element
    pub fn treat_first_ele(&mut self, bi: Handle<T>) -> bool {
        self.ele[0] = Some(Eigen::new(bi, self.kl_dir)?);
        false
    }

    /// Handle second element
    pub fn treat_second_ele(&mut self, bi: Handle<T>) -> Result<bool, ChanException> {
        let ele0 = self.ele[0].as_mut().expect("First element should exist");
        let combine_dir = ele0.try_add(bi.clone(), self.exclude_included, None)?;
        
        if combine_dir != KlineDir::Combine {
            self.ele[1] = Some(Eigen::new(bi, self.kl_dir)?);
            
            if (self.is_up() && self.ele[1].as_ref().unwrap().high < ele0.high) || 
               (self.is_down() && self.ele[1].as_ref().unwrap().low > ele0.low) {
                return Ok(self.reset()?);
            }
        }
        Ok(false)
    }

    /// Handle third element
    pub fn treat_third_ele(&mut self, bi: Handle<T>) -> Result<bool, ChanException> {
        let ele0 = self.ele[0].as_ref().expect("First element should exist");
        let ele1 = self.ele[1].as_mut().expect("Second element should exist");
        
        self.last_evidence_bi = Some(bi.clone());
        
        let allow_top_equal = if self.exclude_included {
            Some(if bi.is_down() { 1 } else { -1 })
        } else {
            None
        };

        let combine_dir = ele1.try_add(bi.clone(), allow_top_equal)?;
        
        if combine_dir == KlineDir::Combine {
            return Ok(false);
        }

        self.ele[2] = Some(Eigen::new(bi, combine_dir)?);
        
        if !self.actual_break()? {
            return Ok(self.reset()?);
        }

        ele1.update_fx(ele0, self.ele[2].as_ref().unwrap(), self.exclude_included, allow_top_equal)?;
        
        let fx = ele1.fx;
        let is_fx = (self.is_up() && fx == FxType::Top) || 
                   (self.is_down() && fx == FxType::Bottom);
        
        Ok(if is_fx { true } else { self.reset()? })
    }

    /// Add a new Bi and return whether a fractal appears
    pub fn add(&mut self, bi: Handle<T>) -> Result<bool, ChanException> {
        assert!(bi.dir() != self.dir, "Bi direction must be different from segment direction");
        
        self.lst.push(bi.clone());
        
        match (self.ele[0].is_none(), self.ele[1].is_none(), self.ele[2].is_none()) {
            (true, _, _) => Ok(self.treat_first_ele(bi)),
            (false, true, _) => self.treat_second_ele(bi),
            (false, false, true) => self.treat_third_ele(bi),
            _ => Err(ChanException::new(
                format!("特征序列3个都找齐了还没处理!! 当前笔:{},当前:{}", bi.idx(), self),
                ErrCode::SegEigenErr,
            )),
        }
    }

    /// Reset the sequence
    pub fn reset(&mut self) -> Result<bool, ChanException> {
        let bi_tmp_list: Vec<Handle<T>> = self.lst[1..].to_vec();
        
        if self.exclude_included {
            self.clear();
            for bi in bi_tmp_list {
                if self.add(bi)? {
                    return Ok(true);
                }
            }
        } else {
            let ele1 = self.ele[1].as_ref().expect("Second element should exist");
            let ele2_begin_idx = ele1.lst[0].idx();
            
            self.ele[0] = self.ele[1].take();
            self.ele[1] = self.ele[2].take();
            self.ele[2] = None;
            
            self.lst = bi_tmp_list
                .into_iter()
                .filter(|bi| bi.idx() >= ele2_begin_idx)
                .collect();
        }
        
        Ok(false)
    }

    /// Check if can be end
    pub fn can_be_end(&mut self, bi_lst: &[Handle<T>]) -> Result<Option<bool>, ChanException> {
        let ele1 = self.ele[1].as_ref().expect("Second element should exist");
        
        if ele1.gap {
            let ele0 = self.ele[0].as_ref().expect("First element should exist");
            let end_bi_idx = self.get_peak_bi_idx()?;
            let thred_value = bi_lst[end_bi_idx].get_end_val();
            let break_thred = if self.is_up() { ele0.low } else { ele0.high };
            
            self.find_revert_fx(bi_lst, end_bi_idx + 2, thred_value, break_thred)
        } else {
            Ok(Some(true))
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

    /// Get peak Bi index
    pub fn get_peak_bi_idx(&self) -> Result<usize, ChanException> {
        self.ele[1].as_ref()
            .expect("Second element should exist")
            .get_peak_bi_idx()
    }

    /// Check if all Bi are sure
    pub fn all_bi_is_sure(&self) -> bool {
        let last_evidence_bi = self.last_evidence_bi.as_ref()
            .expect("Last evidence Bi should exist");
            
        !self.lst.iter().any(|bi| !bi.is_sure()) && last_evidence_bi.is_sure()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.ele = [None, None, None];
        self.lst.clear();
    }

    /// Check for actual break
    pub fn actual_break(&mut self) -> Result<bool, ChanException> {
        if !self.exclude_included {
            return Ok(true);
        }

        let ele2 = self.ele[2].as_ref().expect("Third element should exist");
        let ele1 = self.ele[1].as_ref().expect("Second element should exist");

        if (self.is_up() && ele2.low < ele1.lst.last().unwrap().low()) ||
           (self.is_down() && ele2.high > ele1.lst.last().unwrap().high()) {
            return Ok(true);
        }

        assert_eq!(ele2.lst.len(), 1);
        let ele2_bi = &ele2.lst[0];
        
        if let (Some(next), Some(next_next)) = (ele2_bi.next(), ele2_bi.next().and_then(|bi| bi.next())) {
            if ele2_bi.is_down() && next_next.low() < ele2_bi.low() {
                self.last_evidence_bi = Some(next_next.clone());
                return Ok(true);
            } else if ele2_bi.is_up() && next_next.high() > ele2_bi.high() {
                self.last_evidence_bi = Some(next_next.clone());
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Find reverse fractal
    pub fn find_revert_fx(
        &mut self,
        bi_list: &[Handle<T>],
        begin_idx: usize,
        thred_value: f64,
        break_thred: f64,
    ) -> Result<Option<bool>, ChanException> {
        const COMMON_COMBINE: bool = true;
        
        if begin_idx >= bi_list.len() {
            return Ok(None);
        }

        let first_bi_dir = bi_list[begin_idx].dir();
        let mut eigen_fx = EigenFX::new(
            revert_bi_dir(first_bi_dir),
            !COMMON_COMBINE,
            self.lv,
        );

        // 使用windows(2)来每次取两个元素
        for bi in bi_list[begin_idx..].iter().step_by(2) {
            if eigen_fx.add(bi.clone())? {
                if COMMON_COMBINE {
                    return Ok(Some(true));
                }

                loop {
                    match eigen_fx.can_be_end(bi_list)? {
                        Some(true) | None => {
                            self.last_evidence_bi = Some(bi.clone());
                            return Ok(Some(true));
                        }
                        Some(false) => {
                            if !eigen_fx.reset()? {
                                break;
                            }
                        }
                    }
                }
            }

            if (bi.is_down() && bi.low() < thred_value) || 
               (bi.is_up() && bi.high() > thred_value) {
                return Ok(Some(false));
            }

            if let Some(ele1) = &eigen_fx.ele[1] {
                if (bi.is_down() && ele1.high > break_thred) || 
                   (bi.is_up() && ele1.low < break_thred) {
                    return Ok(Some(true));
                }
            }
        }
        
        Ok(None)
    }
}

impl<T> std::fmt::Display for EigenFX<T>
where T: Clone + std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elements: Vec<String> = self.ele.iter()
            .map(|ele| {
                if let Some(e) = ele {
                    e.lst.iter()
                        .map(|b| b.idx().to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    String::from("[]")
                }
            })
            .collect();
        
        write!(f, "{}", elements.join(" | "))
    }
} 