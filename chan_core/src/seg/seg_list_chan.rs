use crate::common::enums::{BiDir, SegType};
use crate::common::handle::Handle;
use crate::seg::eigen_fx::EigenFx;
use crate::seg::seg_config::SegConfig;
use crate::seg::seg_list_comm::SegListComm;
use crate::traits::bi_trait::BiTrait;

pub struct SegListChan<T: BiTrait> {
    inner: SegListComm<T>,
}

impl<T: BiTrait> SegListChan<T> {
    pub fn new(seg_config: Option<SegConfig>, lv: Option<SegType>) -> Self {
        Self {
            inner: SegListComm::new(seg_config, lv),
        }
    }

    pub fn do_init(&mut self) {
        while !self.inner.lst.is_empty() && !self.inner.lst.last().unwrap().is_sure {
            let seg = self.inner.lst.last().unwrap();
            for bi in &seg.bi_list {
                bi.borrow_mut().parent_seg = None;
            }
            if let Some(pre) = &seg.pre {
                pre.borrow_mut().next = None;
            }
            self.inner.lst.pop();
        }

        if !self.inner.lst.is_empty() {
            let last_seg = self.inner.lst.last().unwrap();
            assert!(last_seg.eigen_fx.is_some() && last_seg.eigen_fx.as_ref().unwrap().ele.last().is_some());
            
            if !last_seg.eigen_fx.as_ref().unwrap().ele.last().unwrap().lst.last().unwrap().is_sure {
                self.inner.lst.pop();
            }
        }
    }

    pub fn update(&mut self, bi_lst: &[Handle<T>]) {
        self.do_init();
        if self.inner.lst.is_empty() {
            self.cal_seg_sure(bi_lst, 0);
        } else {
            let begin_idx = self.inner.lst.last().unwrap().end_bi.borrow().idx + 1;
            self.cal_seg_sure(bi_lst, begin_idx);
        }
        self.inner.collect_left_seg(bi_lst);
    }

    pub fn cal_seg_sure(&mut self, bi_lst: &[Handle<T>], begin_idx: usize) {
        let mut up_eigen = EigenFx::new(BiDir::Up, self.inner.lv);
        let mut down_eigen = EigenFx::new(BiDir::Down, self.inner.lv);
        let mut last_seg_dir = if self.inner.lst.is_empty() {
            None
        } else {
            Some(self.inner.lst.last().unwrap().dir)
        };

        for bi in bi_lst.iter().skip(begin_idx) {
            let mut fx_eigen = None;
            if bi.borrow().is_down() && last_seg_dir != Some(BiDir::Up) {
                if up_eigen.add(bi.clone()) {
                    fx_eigen = Some(&up_eigen);
                }
            } else if bi.borrow().is_up() && last_seg_dir != Some(BiDir::Down) {
                if down_eigen.add(bi.clone()) {
                    fx_eigen = Some(&down_eigen);
                }
            }

            if self.inner.lst.is_empty() {
                if up_eigen.ele[1].is_some() && bi.borrow().is_down() {
                    last_seg_dir = Some(BiDir::Down);
                    down_eigen.clear();
                } else if down_eigen.ele[1].is_some() && bi.borrow().is_up() {
                    up_eigen.clear();
                    last_seg_dir = Some(BiDir::Up);
                }

                if up_eigen.ele[1].is_none() && last_seg_dir == Some(BiDir::Down) && bi.borrow().dir == BiDir::Down {
                    last_seg_dir = None;
                } else if down_eigen.ele[1].is_none() && last_seg_dir == Some(BiDir::Up) && bi.borrow().dir == BiDir::Up {
                    last_seg_dir = None;
                }
            }

            if let Some(eigen) = fx_eigen {
                self.treat_fx_eigen(eigen, bi_lst);
                break;
            }
        }
    }

    pub fn treat_fx_eigen(&mut self, fx_eigen: &EigenFx<T>, bi_lst: &[Handle<T>]) {
        let test = fx_eigen.can_be_end(bi_lst);
        let end_bi_idx = fx_eigen.get_peak_bi_idx();
        
        match test {
            Some(true) | None => {
                let is_true = test.is_some();
                if !self.inner.add_new_seg(bi_lst, end_bi_idx, is_true && fx_eigen.all_bi_is_sure()) {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                    return;
                }
                self.inner.lst.last_mut().unwrap().eigen_fx = Some(fx_eigen.clone());
                if is_true {
                    self.cal_seg_sure(bi_lst, end_bi_idx + 1);
                }
            }
            Some(false) => {
                self.cal_seg_sure(bi_lst, fx_eigen.lst[1].borrow().idx);
            }
        }
    }
} 