use std::collections::HashMap;
use crate::bi::bi::Bi;
use crate::bi::bi_list::BiList;
use crate::common::enums::BspType;
use crate::common::handle::Handle;
use crate::common::utils::has_overlap;
use crate::seg::seg::Seg;
use crate::seg::seg_list_comm::SegListComm;
use crate::traits::line_trait::LineTrait;
use crate::zs::zs::ZS;
use super::bs_point::BSPoint;
use super::bs_point_config::{BSPointConfig, PointConfig};

pub struct BSPointList<T: LineTrait> {
    pub lst: Vec<Handle<BSPoint<T>>>,
    pub bsp_dict: HashMap<usize, Handle<BSPoint<T>>>,
    pub bsp1_lst: Vec<Handle<BSPoint<T>>>,
    pub config: BSPointConfig,
    pub last_sure_pos: isize,
}

impl<T: LineTrait> BSPointList<T> {
    pub fn new(bs_point_config: BSPointConfig) -> Self {
        Self {
            lst: Vec::new(),
            bsp_dict: HashMap::new(),
            bsp1_lst: Vec::new(),
            config: bs_point_config,
            last_sure_pos: -1,
        }
    }

    pub fn len(&self) -> usize {
        self.lst.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Handle<BSPoint<T>>> {
        self.lst.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Handle<BSPoint<T>>> {
        self.lst.get(index)
    }

    pub fn cal(&mut self, bi_list: &BiList<T>, seg_list: &SegListComm<T>) {
        self.lst.retain(|bsp| bsp.borrow().klu.borrow().idx as isize <= self.last_sure_pos);
        self.bsp_dict = self.lst.iter()
            .map(|bsp| (bsp.borrow().bi.borrow().get_end_klu().borrow().idx, bsp.clone()))
            .collect();
        self.bsp1_lst.retain(|bsp| bsp.borrow().klu.borrow().idx as isize <= self.last_sure_pos);

        self.cal_seg_bs1point(seg_list, bi_list);
        self.cal_seg_bs2point(seg_list, bi_list);
        self.cal_seg_bs3point(seg_list, bi_list);

        self.update_last_pos(seg_list);
    }

    pub fn update_last_pos(&mut self, seg_list: &SegListComm<T>) {
        self.last_sure_pos = -1;
        for seg in seg_list.iter().rev() {
            if seg.borrow().is_sure {
                self.last_sure_pos = seg.borrow().end_bi.borrow().get_begin_klu().borrow().idx as isize;
                return;
            }
        }
    }

    pub fn seg_need_cal(&self, seg: &Handle<Seg<T>>) -> bool {
        seg.borrow().end_bi.borrow().get_end_klu().borrow().idx as isize > self.last_sure_pos
    }

    pub fn add_bs(
        &mut self,
        bs_type: BspType,
        bi: Handle<T>,
        relate_bsp1: Option<Handle<BSPoint<T>>>,
        is_target_bsp: bool,
        feature_dict: Option<HashMap<String, f64>>,
    ) {
        let is_buy = bi.borrow().is_down();
        let end_klu_idx = bi.borrow().get_end_klu().borrow().idx;

        if let Some(exist_bsp) = self.bsp_dict.get(&end_klu_idx) {
            assert_eq!(exist_bsp.borrow().is_buy, is_buy);
            exist_bsp.borrow_mut().add_another_bsp_prop(bs_type, relate_bsp1);
            if let Some(features) = feature_dict {
                exist_bsp.borrow_mut().add_feat(features, None);
            }
            return;
        }

        if !self.config.get_bs_config(is_buy).target_types.contains(&bs_type) {
            is_target_bsp = false;
        }

        if is_target_bsp || matches!(bs_type, BspType::BS1 | BspType::BS1Peak) {
            let bsp = Handle::new(BSPoint::new(
                bi.clone(),
                is_buy,
                bs_type,
                relate_bsp1,
                feature_dict,
            ));

            if is_target_bsp {
                self.lst.push(bsp.clone());
                self.bsp_dict.insert(end_klu_idx, bsp.clone());
            }
            if matches!(bs_type, BspType::BS1 | BspType::BS1Peak) {
                self.bsp1_lst.push(bsp);
            }
        }
    }

    pub fn cal_seg_bs1point(&mut self, seg_list: &SegListComm<T>, bi_list: &BiList<T>) {
        for seg in seg_list.iter() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            self.cal_single_bs1point(seg, bi_list);
        }
    }

    pub fn cal_single_bs1point(&mut self, seg: &Handle<Seg<T>>, bi_list: &BiList<T>) {
        let seg_ref = seg.borrow();
        let bsp_conf = self.config.get_bs_config(seg_ref.is_down());
        
        let zs_cnt = if bsp_conf.bsp1_only_multibi_zs {
            seg_ref.get_multi_bi_zs_cnt()
        } else {
            seg_ref.zs_lst.len()
        };
        
        let is_target_bsp = bsp_conf.min_zs_cnt <= 0 || zs_cnt >= bsp_conf.min_zs_cnt as usize;

        if !seg_ref.zs_lst.is_empty() {
            let last_zs = &seg_ref.zs_lst.last().unwrap();
            let valid_last_zs = !last_zs.is_one_bi_zs() && 
                ((last_zs.bi_out.is_some() && last_zs.bi_out.as_ref().unwrap().borrow().idx >= seg_ref.end_bi.borrow().idx) ||
                last_zs.bi_lst.last().unwrap().borrow().idx >= seg_ref.end_bi.borrow().idx) &&
                seg_ref.end_bi.borrow().idx - last_zs.get_bi_in().borrow().idx > 2;

            if valid_last_zs {
                self.treat_bsp1(seg, bsp_conf, is_target_bsp);
            } else {
                self.treat_pz_bsp1(seg, bsp_conf, bi_list, is_target_bsp);
            }
        }
    }

    fn treat_bsp1(&mut self, seg: &Handle<Seg<T>>, bsp_conf: &PointConfig, mut is_target_bsp: bool) {
        let seg_ref = seg.borrow();
        let last_zs = seg_ref.zs_lst.last().unwrap();
        let (break_peak, _) = last_zs.out_bi_is_peak(seg_ref.end_bi.borrow().idx);
        
        if bsp_conf.bs1_peak && !break_peak {
            is_target_bsp = false;
        }
        
        let (is_diver, divergence_rate) = last_zs.is_divergence(bsp_conf, Some(&seg_ref.end_bi));
        if !is_diver {
            is_target_bsp = false;
        }

        let mut feature_dict = HashMap::new();
        feature_dict.insert("bsp1_zs_height".to_string(), (last_zs.high.unwrap() - last_zs.low.unwrap()) / last_zs.low.unwrap());
        feature_dict.insert("bsp1_bi_amp".to_string(), seg_ref.end_bi.borrow().amp());
        if let Some(rate) = divergence_rate {
            feature_dict.insert("bsp1_divergence_rate".to_string(), rate);
        }

        self.add_bs(
            if bsp_conf.bs1_peak { BspType::BS1Peak } else { BspType::BS1 },
            seg_ref.end_bi.clone(),
            None,
            is_target_bsp,
            Some(feature_dict),
        );
    }

    fn treat_pz_bsp1(
        &mut self,
        seg: &Handle<Seg<T>>,
        bsp_conf: &PointConfig,
        bi_list: &BiList<T>,
        is_target_bsp: bool,
    ) {
        let seg_ref = seg.borrow();
        if seg_ref.bi_list.len() < 3 {
            return;
        }

        let mut feature_dict = HashMap::new();
        feature_dict.insert("bsp1_bi_amp".to_string(), seg_ref.end_bi.borrow().amp());

        self.add_bs(
            if bsp_conf.bs1_peak { BspType::BS1Peak } else { BspType::BS1 },
            seg_ref.end_bi.clone(),
            None,
            is_target_bsp,
            Some(feature_dict),
        );
    }

    pub fn cal_seg_bs2point(&mut self, seg_list: &SegListComm<T>, bi_list: &BiList<T>) {
        let mut bsp1_bi_idx_dict = HashMap::new();
        for bsp in &self.bsp1_lst {
            bsp1_bi_idx_dict.insert(bsp.borrow().bi.borrow().idx, bsp.clone());
        }

        for (i, seg) in seg_list.iter().enumerate() {
            if !self.seg_need_cal(seg) {
                continue;
            }
            self.cal_single_bs2point(seg, bi_list, &bsp1_bi_idx_dict);
            if self.config.b_conf.bsp2s_follow_2 || self.config.s_conf.bsp2s_follow_2 {
                self.cal_single_bs2s_point(seg, bi_list, &bsp1_bi_idx_dict);
            }
        }
    }

    fn cal_single_bs2point(
        &mut self,
        seg: &Handle<Seg<T>>,
        bi_list: &BiList<T>,
        bsp1_bi_idx_dict: &HashMap<usize, Handle<BSPoint<T>>>,
    ) {
        let seg_ref = seg.borrow();
        let bsp_conf = self.config.get_bs_config(seg_ref.is_down());
        
        if !bsp_conf.bsp2_follow_1 {
            return;
        }

        let mut last_bsp1_bi_idx = None;
        for zs in &seg_ref.zs_lst {
            if let Some(bi_out) = &zs.bi_out {
                let bi_out_idx = bi_out.borrow().idx;
                if bsp1_bi_idx_dict.contains_key(&bi_out_idx) {
                    last_bsp1_bi_idx = Some(bi_out_idx);
                }
            }
        }

        if let Some(last_idx) = last_bsp1_bi_idx {
            let real_bsp1 = bsp1_bi_idx_dict.get(&last_idx).unwrap();
            let mut feature_dict = HashMap::new();
            feature_dict.insert("bsp2_bi_amp".to_string(), seg_ref.end_bi.borrow().amp());
            
            self.add_bs(
                BspType::BS2,
                seg_ref.end_bi.clone(),
                Some(real_bsp1.clone()),
                true,
                Some(feature_dict),
            );
        }
    }

    fn cal_single_bs2s_point(
        &mut self,
        seg: &Handle<Seg<T>>,
        bi_list: &BiList<T>,
        bsp1_bi_idx_dict: &HashMap<usize, Handle<BSPoint<T>>>,
    ) {
        let seg_ref = seg.borrow();
        let bsp_conf = self.config.get_bs_config(seg_ref.is_down());
        
        if !bsp_conf.bsp2s_follow_2 {
            return;
        }

        let mut last_bsp2_bi = None;
        let mut last_bsp2_real_bsp1 = None;
        
        for bi in &seg_ref.bi_list {
            let bi_idx = bi.borrow().idx;
            if let Some(bsp) = self.bsp_dict.get(&bi.borrow().get_end_klu().borrow().idx) {
                if bsp.borrow().bs_type.contains(&BspType::BS2) {
                    last_bsp2_bi = Some(bi.clone());
                    last_bsp2_real_bsp1 = bsp.borrow().relate_bsp1.clone();
                }
            }
        }

        if let (Some(bsp2_bi), Some(real_bsp1)) = (last_bsp2_bi, last_bsp2_real_bsp1) {
            if bsp2s_break_bsp1(&seg_ref.end_bi, &bsp2_bi) {
                let mut feature_dict = HashMap::new();
                feature_dict.insert("bsp2s_bi_amp".to_string(), seg_ref.end_bi.borrow().amp());
                
                self.add_bs(
                    BspType::BS2Strict,
                    seg_ref.end_bi.clone(),
                    Some(real_bsp1),
                    true,
                    Some(feature_dict),
                );
            }
        }
    }

    pub fn cal_seg_bs3point(&mut self, seg_list: &SegListComm<T>, bi_list: &BiList<T>) {
        let mut bsp1_bi_idx_dict = HashMap::new();
        for bsp in &self.bsp1_lst {
            bsp1_bi_idx_dict.insert(bsp.borrow().bi.borrow().idx, bsp.clone());
        }

        for (i, seg) in seg_list.iter().enumerate() {
            if !self.seg_need_cal(seg) {
                continue;
            }

            let seg_ref = seg.borrow();
            let bsp1_bi = if let Some(last_zs) = seg_ref.zs_lst.last() {
                last_zs.bi_out.clone()
            } else {
                None
            };

            let (bsp_conf, next_seg, next_seg_idx, real_bsp1, bsp1_bi_idx) = if let Some(bsp1_bi) = &bsp1_bi {
                let bsp_conf = self.config.get_bs_config(seg_ref.is_down());
                let real_bsp1 = bsp1_bi_idx_dict.get(&bsp1_bi.borrow().idx).cloned();
                let next_seg_idx = seg_ref.idx + 1;
                (bsp_conf, seg_ref.next.clone(), next_seg_idx, real_bsp1, bsp1_bi.borrow().idx)
            } else {
                let bsp_conf = self.config.get_bs_config(seg_ref.is_up());
                (bsp_conf, Some(seg.clone()), seg_ref.idx, None, -1)
            };

            if bsp_conf.bsp3_follow_1 && !self.bsp_dict.values().any(|bsp| bsp.borrow().bi.borrow().idx == bsp1_bi_idx) {
                continue;
            }

            if let Some(next_seg) = &next_seg {
                self.treat_bsp3_after(
                    seg_list,
                    next_seg,
                    bsp_conf,
                    bi_list,
                    real_bsp1.clone(),
                    bsp1_bi_idx,
                    next_seg_idx,
                );
            }

            self.treat_bsp3_before(
                seg_list,
                seg,
                next_seg.as_ref(),
                bsp1_bi,
                bsp_conf,
                bi_list,
                real_bsp1,
                next_seg_idx,
            );
        }
    }

    fn treat_bsp3_after(
        &mut self,
        seg_list: &SegListComm<T>,
        next_seg: &Handle<Seg<T>>,
        bsp_conf: &PointConfig,
        bi_list: &BiList<T>,
        real_bsp1: Option<Handle<BSPoint<T>>>,
        bsp1_bi_idx: isize,
        next_seg_idx: usize,
    ) {
        let next_seg_ref = next_seg.borrow();
        if next_seg_ref.bi_list.len() < 3 {
            return;
        }

        let first_zs = if let Some(zs) = next_seg_ref.zs_lst.first() {
            if zs.is_one_bi_zs() {
                return;
            }
            zs
        } else {
            return;
        };

        let end_bi_idx = cal_bsp3_bi_end_idx(Some(next_seg));
        for bsp3_bi in bi_list.iter().skip(bsp1_bi_idx as usize + 2).step_by(2) {
            let bsp3_bi_ref = bsp3_bi.borrow();
            if bsp3_bi_ref.idx as f64 > end_bi_idx {
                break;
            }
            if bsp3_bi_ref.seg_idx.unwrap() != next_seg_idx && next_seg_idx < seg_list.len() - 2 {
                return;
            }
            if bsp3_back2zs(bsp3_bi, first_zs) {
                return;
            }
            let bsp3_peak_zs = bsp3_break_zspeak(bsp3_bi, first_zs);
            if bsp_conf.bsp3_peak && !bsp3_peak_zs {
                return;
            }

            let mut feature_dict = HashMap::new();
            feature_dict.insert(
                "bsp3_zs_height".to_string(),
                (first_zs.high.unwrap() - first_zs.low.unwrap()) / first_zs.low.unwrap(),
            );
            feature_dict.insert("bsp3_bi_amp".to_string(), bsp3_bi_ref.amp());

            self.add_bs(
                BspType::BS3,
                bsp3_bi.clone(),
                real_bsp1.clone(),
                true,
                Some(feature_dict),
            );
        }
    }

    fn treat_bsp3_before(
        &mut self,
        seg_list: &SegListComm<T>,
        seg: &Handle<Seg<T>>,
        next_seg: Option<&Handle<Seg<T>>>,
        bsp1_bi: Option<Handle<T>>,
        bsp_conf: &PointConfig,
        bi_list: &BiList<T>,
        real_bsp1: Option<Handle<BSPoint<T>>>,
        next_seg_idx: usize,
    ) {
        let seg_ref = seg.borrow();
        let cmp_zs = seg_ref.get_final_multi_bi_zs();
        if cmp_zs.is_none() || bsp1_bi.is_none() {
            return;
        }
        let cmp_zs = cmp_zs.unwrap();
        let bsp1_bi = bsp1_bi.unwrap();

        if bsp_conf.strict_bsp3 {
            if cmp_zs.bi_out.is_none() || cmp_zs.bi_out.as_ref().unwrap().borrow().idx != bsp1_bi.borrow().idx {
                return;
            }
        }

        let end_bi_idx = cal_bsp3_bi_end_idx(next_seg);
        for bsp3_bi in bi_list.iter().skip(bsp1_bi.borrow().idx as usize + 2).step_by(2) {
            let bsp3_bi_ref = bsp3_bi.borrow();
            if bsp3_bi_ref.idx as f64 > end_bi_idx {
                break;
            }
            
            assert!(bsp3_bi_ref.seg_idx.is_some());
            if bsp3_bi_ref.seg_idx.unwrap() != next_seg_idx && bsp3_bi_ref.seg_idx.unwrap() < seg_list.len() - 1 {
                break;
            }
            
            if bsp3_back2zs(bsp3_bi, &cmp_zs) {
                continue;
            }

            let mut feature_dict = HashMap::new();
            feature_dict.insert(
                "bsp3_zs_height".to_string(),
                (cmp_zs.high.unwrap() - cmp_zs.low.unwrap()) / cmp_zs.low.unwrap(),
            );
            feature_dict.insert("bsp3_bi_amp".to_string(), bsp3_bi_ref.amp());

            self.add_bs(
                BspType::BS3Peak,
                bsp3_bi.clone(),
                real_bsp1.clone(),
                true,
                Some(feature_dict),
            );
            break;
        }
    }

    pub fn get_lastest_bsp_list(&self) -> Vec<Handle<BSPoint<T>>> {
        if self.lst.is_empty() {
            return vec![];
        }
        let mut result = self.lst.clone();
        result.sort_by(|a, b| {
            b.borrow().bi.borrow().idx.cmp(&a.borrow().bi.borrow().idx)
        });
        result
    }
}

// Helper functions
pub fn bsp2s_break_bsp1<T: LineTrait>(bsp2s_bi: &Handle<T>, bsp2_break_bi: &Handle<T>) -> bool {
    let bsp2s = bsp2s_bi.borrow();
    let break_bi = bsp2_break_bi.borrow();
    (bsp2s.is_down() && bsp2s._low() < break_bi._low()) ||
    (bsp2s.is_up() && bsp2s._high() > break_bi._high())
}

pub fn bsp3_back2zs<T: LineTrait>(bsp3_bi: &Handle<T>, zs: &ZS<T>) -> bool {
    let bi = bsp3_bi.borrow();
    (bi.is_down() && bi._low() < zs.high.unwrap()) ||
    (bi.is_up() && bi._high() > zs.low.unwrap())
}

pub fn bsp3_break_zspeak<T: LineTrait>(bsp3_bi: &Handle<T>, zs: &ZS<T>) -> bool {
    let bi = bsp3_bi.borrow();
    (bi.is_down() && bi._high() >= zs.peak_high) ||
    (bi.is_up() && bi._low() <= zs.peak_low)
}

pub fn cal_bsp3_bi_end_idx<T: LineTrait>(seg: Option<&Handle<Seg<T>>>) -> f64 {
    match seg {
        None => f64::INFINITY,
        Some(seg) => {
            let seg = seg.borrow();
            if seg.get_multi_bi_zs_cnt() == 0 && seg.next.is_none() {
                f64::INFINITY
            } else {
                let mut end_bi_idx = seg.end_bi.borrow().idx as f64 - 1.0;
                for zs in &seg.zs_lst {
                    if zs.is_one_bi_zs() {
                        continue;
                    }
                    if let Some(bi_out) = &zs.bi_out {
                        end_bi_idx = bi_out.borrow().idx as f64;
                        break;
                    }
                }
                end_bi_idx
            }
        }
    }
} 