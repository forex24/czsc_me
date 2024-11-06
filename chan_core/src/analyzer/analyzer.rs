use std::collections::HashMap;
use std::path::Path;
use polars::prelude::*;
use crate::common::{
    chan_exception::{ChanException, ErrCode},
    chan_config::ChanConfig,
    enums::{SegType, KlineDir},
    time::Time,
};
use crate::bi::{bi::Bi, bi_list::BiList};
use crate::seg::{
    seg::Seg,
    seg_config::SegConfig,
    seg_list_comm::SegListComm,
};
use crate::zs::zs_list::ZSList;
use crate::bs_point::bs_point_list::BSPointList;
use crate::kline::{kline_list::KLineList, kline_unit::KLineUnit};
use crate::metric::metric_model::MetricModel;

/// 分析器，负责处理笔、线段、中枢和买卖点的计算
#[derive(Debug)]
pub struct Analyzer {
    pub kline_list: KLineList,
    pub bi_list: BiList,
    pub seg_list: Box<dyn SegListComm<Item = Bi>>,
    pub segseg_list: Box<dyn SegListComm<Item = Seg<Bi>>>,
    pub zs_list: ZSList,
    pub segzs_list: ZSList,
    pub bs_point_lst: BSPointList<Bi, BiList>,
    pub seg_bs_point_lst: BSPointList<Seg<Bi>, Box<dyn SegListComm<Item = Bi>>>,
    pub metric_model_lst: Vec<Box<dyn MetricModel>>,
    pub step_calculation: bool,
    pub bs_point_history: Vec<HashMap<String, String>>,
    pub seg_bs_point_history: Vec<HashMap<String, String>>,
    config: ChanConfig,
}

impl Analyzer {
    /// Create a new Analyzer instance
    pub fn new(kl_type: String, conf: ChanConfig) -> Result<Self, ChanException> {
        let seg_list = get_seglist_instance(&conf.seg_conf, SegType::Bi)?;
        let segseg_list = get_seglist_instance(&conf.seg_conf, SegType::Seg)?;

        Ok(Self {
            kline_list: KLineList::new(kl_type),
            bi_list: BiList::new(conf.bi_conf.clone()),
            seg_list,
            segseg_list,
            zs_list: ZSList::new(conf.zs_conf.clone()),
            segzs_list: ZSList::new(conf.zs_conf.clone()),
            bs_point_lst: BSPointList::new(conf.bs_point_conf.clone()),
            seg_bs_point_lst: BSPointList::new(conf.seg_bs_point_conf.clone()),
            metric_model_lst: conf.get_metric_model(),
            step_calculation: conf.trigger_step,
            config: conf,
            bs_point_history: Vec::new(),
            seg_bs_point_history: Vec::new(),
        })
    }

    /// Deep clone the analyzer
    pub fn deep_clone(&self) -> Result<Self, ChanException> {
        Ok(Self {
            kline_list: self.kline_list.deep_clone(),
            bi_list: self.bi_list.clone(),
            seg_list: self.seg_list.clone_box(),
            segseg_list: self.segseg_list.clone_box(),
            zs_list: self.zs_list.clone(),
            segzs_list: self.segzs_list.clone(),
            bs_point_lst: self.bs_point_lst.clone(),
            seg_bs_point_lst: self.seg_bs_point_lst.clone(),
            metric_model_lst: self.metric_model_lst.clone(),
            step_calculation: self.step_calculation,
            config: self.config.clone(),
            bs_point_history: self.bs_point_history.clone(),
            seg_bs_point_history: self.seg_bs_point_history.clone(),
        })
    }

    /// Calculate segments
    fn cal_seg(&mut self) -> Result<(), ChanException> {
        let mut sure_seg_cnt = 0;
        if self.seg_list.len() == 0 {
            for bi in self.bi_list.iter_mut() {
                bi.set_seg_idx(0);
            }
            return Ok(());
        }

        let mut begin_seg = self.seg_list.last().unwrap();
        for seg in self.seg_list.iter().rev() {
            if seg.is_sure() {
                sure_seg_cnt += 1;
            } else {
                sure_seg_cnt = 0;
            }
            begin_seg = seg;
            if sure_seg_cnt > 2 {
                break;
            }
        }

        let mut cur_seg = self.seg_list.last().unwrap();
        for bi in self.bi_list.iter_mut().rev() {
            if bi.seg_idx().is_some() && bi.idx() < begin_seg.start_bi().idx() {
                break;
            }
            if bi.idx() > cur_seg.end_bi().idx() {
                bi.set_seg_idx(cur_seg.idx() + 1);
                continue;
            }
            if bi.idx() < cur_seg.start_bi().idx() {
                cur_seg = cur_seg.pre().expect("cur_seg.pre should exist");
            }
            bi.set_seg_idx(cur_seg.idx());
        }

        Ok(())
    }

    /// Update ZhongShu in segments
    fn update_zs_in_seg(&mut self, is_seg_level: bool) -> Result<(), ChanException> {
        let (seg_list, zs_list) = if is_seg_level {
            (&mut self.segseg_list, &mut self.segzs_list)
        } else {
            (&mut self.seg_list, &mut self.zs_list)
        };

        let mut sure_seg_cnt = 0;
        for seg in seg_list.iter_mut().rev() {
            if seg.ele_inside_is_sure() {
                break;
            }
            if seg.is_sure() {
                sure_seg_cnt += 1;
            }
            seg.clear_zs_lst();

            for zs in zs_list.iter_mut().rev() {
                if zs.end().idx() < seg.start_bi().get_begin_klu().idx() {
                    break;
                }
                if zs.is_inside(seg) {
                    seg.add_zs(zs);
                }
                assert!(zs.begin_bi().idx() > 0);
                
                if is_seg_level {
                    zs.set_bi_in(&self.seg_list[zs.begin_bi().idx() - 1]);
                    if zs.end_bi().idx() + 1 < self.seg_list.len() {
                        zs.set_bi_out(&self.seg_list[zs.end_bi().idx() + 1]);
                    }
                    zs.set_bi_lst(&self.seg_list[zs.begin_bi().idx()..=zs.end_bi().idx()]);
                } else {
                    zs.set_bi_in(&self.bi_list[zs.begin_bi().idx() - 1]);
                    if zs.end_bi().idx() + 1 < self.bi_list.len() {
                        zs.set_bi_out(&self.bi_list[zs.end_bi().idx() + 1]);
                    }
                    zs.set_bi_lst(&self.bi_list[zs.begin_bi().idx()..=zs.end_bi().idx()]);
                }
            }

            if sure_seg_cnt > 2 && !seg.ele_inside_is_sure() {
                seg.set_ele_inside_is_sure(true);
            }
        }
        Ok(())
    }

    /// Record current buy/sell points
    fn record_current_bs_points(&mut self) {
        // Record latest bs_points
        if let Some(latest_bsp) = self.bs_point_lst.last() {
            let mut record = HashMap::new();
            record.insert("begin_time".to_string(), latest_bsp.klu.time.to_string());
            record.insert("bsp_type".to_string(), latest_bsp.type2str());
            record.insert("is_buy".to_string(), latest_bsp.is_buy.to_string());
            if let Some(relate_bsp1) = &latest_bsp.relate_bsp1 {
                record.insert("relate_bsp1".to_string(), relate_bsp1.klu.time.to_string());
            }
            if let Some(bi) = &latest_bsp.bi {
                record.insert("bi_idx".to_string(), bi.idx().to_string());
                record.insert("bi_begin_time".to_string(), bi.get_begin_klu().time.to_string());
                record.insert("bi_end_time".to_string(), bi.get_end_klu().time.to_string());
            }
            self.bs_point_history.push(record);
        }

        // Record latest seg_bs_points
        if let Some(latest_seg_bsp) = self.seg_bs_point_lst.last() {
            let mut record = HashMap::new();
            record.insert("begin_time".to_string(), latest_seg_bsp.klu.time.to_string());
            record.insert("bsp_type".to_string(), latest_seg_bsp.type2str());
            record.insert("is_buy".to_string(), latest_seg_bsp.is_buy.to_string());
            if let Some(relate_bsp1) = &latest_seg_bsp.relate_bsp1 {
                record.insert("relate_bsp1".to_string(), relate_bsp1.klu.time.to_string());
            }
            if let Some(bi) = &latest_seg_bsp.bi {
                record.insert("seg_idx".to_string(), bi.idx().to_string());
                record.insert("bi_begin_time".to_string(), bi.get_begin_klu().time.to_string());
                record.insert("bi_end_time".to_string(), bi.get_end_klu().time.to_string());
            }
            self.seg_bs_point_history.push(record);
        }
    }

    /// Convert analysis results to DataFrames
    pub fn to_dataframes(&self) -> Result<HashMap<String, DataFrame>, ChanException> {
        let mut dataframes = HashMap::new();

        // K线数据
        let kline_data = df!(
            "time" => self.kline_list.lst.iter().map(|k| k.time_begin().to_string()).collect::<Vec<_>>(),
            "high" => self.kline_list.lst.iter().map(|k| k.high()).collect::<Vec<_>>(),
            "low" => self.kline_list.lst.iter().map(|k| k.low()).collect::<Vec<_>>(),
            "open" => self.kline_list.lst.iter().map(|k| k.lst[0].open).collect::<Vec<_>>(),
            "close" => self.kline_list.lst.iter().map(|k| k.lst[0].close).collect::<Vec<_>>(),
            "volume" => self.kline_list.lst.iter().map(|k| k.lst[0].trade_info.volume).collect::<Vec<_>>(),
            "amount" => self.kline_list.lst.iter().map(|k| k.lst[0].trade_info.amount).collect::<Vec<_>>(),
            "fx_type" => self.kline_list.lst.iter().map(|k| k.fx().to_string()).collect::<Vec<_>>()
        )?;
        dataframes.insert("klines".to_string(), kline_data);

        // 笔数据
        if !self.bi_list.is_empty() {
            let bi_data = df!(
                "idx" => self.bi_list.iter().map(|b| b.idx()).collect::<Vec<_>>(),
                "begin_time" => self.bi_list.iter().map(|b| b.get_begin_klu().time.to_string()).collect::<Vec<_>>(),
                "end_time" => self.bi_list.iter().map(|b| b.get_end_klu().time.to_string()).collect::<Vec<_>>(),
                "high" => self.bi_list.iter().map(|b| b.high()).collect::<Vec<_>>(),
                "low" => self.bi_list.iter().map(|b| b.low()).collect::<Vec<_>>(),
                "direction" => self.bi_list.iter().map(|b| b.direction().to_string()).collect::<Vec<_>>()
            )?;
            dataframes.insert("bis".to_string(), bi_data);
        }

        // 线段数据
        if !self.seg_list.is_empty() {
            let seg_data = df!(
                "idx" => self.seg_list.iter().map(|s| s.idx()).collect::<Vec<_>>(),
                "begin_time" => self.seg_list.iter().map(|s| s.start_bi().get_begin_klu().time.to_string()).collect::<Vec<_>>(),
                "end_time" => self.seg_list.iter().map(|s| s.end_bi().get_end_klu().time.to_string()).collect::<Vec<_>>(),
                "high" => self.seg_list.iter().map(|s| s.high()).collect::<Vec<_>>(),
                "low" => self.seg_list.iter().map(|s| s.low()).collect::<Vec<_>>(),
                "direction" => self.seg_list.iter().map(|s| s.direction().to_string()).collect::<Vec<_>>()
            )?;
            dataframes.insert("segments".to_string(), seg_data);
        }

        // 买卖点历史
        if !self.bs_point_history.is_empty() {
            let bs_history = DataFrame::new(vec![
                Series::new("begin_time", self.bs_point_history.iter().map(|h| h.get("begin_time").unwrap()).collect::<Vec<_>>()),
                Series::new("bsp_type", self.bs_point_history.iter().map(|h| h.get("bsp_type").unwrap()).collect::<Vec<_>>()),
                Series::new("is_buy", self.bs_point_history.iter().map(|h| h.get("is_buy").unwrap()).collect::<Vec<_>>()),
                Series::new("relate_bsp1", self.bs_point_history.iter().map(|h| h.get("relate_bsp1").unwrap_or(&"".to_string())).collect::<Vec<_>>()),
                Series::new("bi_idx", self.bs_point_history.iter().map(|h| h.get("bi_idx").unwrap_or(&"".to_string())).collect::<Vec<_>>()),
                Series::new("bi_begin_time", self.bs_point_history.iter().map(|h| h.get("bi_begin_time").unwrap_or(&"".to_string())).collect::<Vec<_>>()),
                Series::new("bi_end_time", self.bs_point_history.iter().map(|h| h.get("bi_end_time").unwrap_or(&"".to_string())).collect::<Vec<_>>())
            ])?;
            dataframes.insert("bs_point_history".to_string(), bs_history);
        }

        Ok(dataframes)
    }

    /// Save analysis results to CSV files
    pub fn to_csv(&self, directory: &str) -> Result<(), ChanException> {
        std::fs::create_dir_all(directory)?;
        
        let dataframes = self.to_dataframes()?;
        
        for (name, df) in dataframes {
            let file_path = Path::new(directory).join(format!("{}.csv", name));
            df.write_csv(&file_path)?;
            println!("Saved {} to {}", name, file_path.display());
        }
        
        Ok(())
    }
}

/// Get segment list instance based on configuration
fn get_seglist_instance(seg_config: &SegConfig, lv: SegType) -> Result<Box<dyn SegListComm>, ChanException> {
    match seg_config.seg_algo.as_str() {
        "chan" => {
            use crate::seg::seg_list_chan::SegListChan;
            Ok(Box::new(SegListChan::new(seg_config.clone(), lv)))
        },
        "1+1" => {
            println!("Please avoid using seg_algo=1+1 as it is deprecated and no longer maintained.");
            use crate::seg::seg_list_dyh::SegListDYH;
            Ok(Box::new(SegListDYH::new(seg_config.clone(), lv)))
        },
        "break" => {
            println!("Please avoid using seg_algo=break as it is deprecated and no longer maintained.");
            use crate::seg::seg_list_def::SegListDef;
            Ok(Box::new(SegListDef::new(seg_config.clone(), lv)))
        },
        _ => Err(ChanException::new(
            format!("unsupport seg algorithm:{}", seg_config.seg_algo),
            ErrCode::ParaError,
        )),
    }
} 