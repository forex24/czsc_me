use std::collections::HashMap;
use crate::common::{
    data_field::DataField,
    trend_type::TrendType,
    time::Time,
    chan_exception::{ChanException, ErrCode},
    handle::{Handle, Indexable, AsHandle},
    impl_handle,
};
use crate::math::{
    boll::{BollMetric, BollModel},
    demark::{DemarkEngine, DemarkIndex},
    kdj::KDJ,
    macd::{MACD, MACDItem},
    rsi::RSI,
    trend_model::TrendModel,
};
use crate::kline::trade_info::TradeInfo;

#[derive(Debug)]
pub struct KLineUnit {
    handle: Handle<Self>,
    pub kl_type: Option<String>, // TODO: Consider making this an enum
    pub time: Time,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub trade_info: TradeInfo,
    pub demark: DemarkIndex,
    pub sub_kl_list: Vec<KLineUnit>,
    pub sup_kl: Option<Box<KLineUnit>>,
    pub trend: HashMap<TrendType, HashMap<i32, f64>>,
    pub limit_flag: i32,
    // Optional fields that are set via set_metric
    pub macd: Option<MACDItem>,
    pub boll: Option<BollMetric>,
    pub rsi: Option<f64>,
    pub kdj: Option<KDJ>,
}

// Automatically implement AsHandle and Indexable traits
impl_handle!(KLineUnit);

impl KLineUnit {
    #[allow(clippy::borrowed_box)]
    pub fn new(
        box_vec: &Box<Vec<Self>>,
        index: usize,
        kl_dict: &HashMap<DataField, f64>, 
        autofix: bool
    ) -> Result<Self, ChanException> {
        let mut unit = Self {
            handle: Handle::new(box_vec, index),
            kl_type: None,
            time: Time::from_dict(kl_dict),
            close: *kl_dict.get(&DataField::FieldClose).unwrap(),
            open: *kl_dict.get(&DataField::FieldOpen).unwrap(),
            high: *kl_dict.get(&DataField::FieldHigh).unwrap(),
            low: *kl_dict.get(&DataField::FieldLow).unwrap(),
            trade_info: TradeInfo::new(kl_dict),
            demark: DemarkIndex::default(),
            sub_kl_list: Vec::new(),
            sup_kl: None,
            trend: HashMap::new(),
            limit_flag: 0,
            macd: None,
            boll: None,
            rsi: None,
            kdj: None,
        };

        unit.check(autofix)?;
        Ok(unit)
    }

    // Get previous KLineUnit using Handle
    pub fn prev(&self) -> Option<&Self> {
        self.handle.prev().map(|h| h.to_ref())
    }

    // Get next KLineUnit using Handle
    pub fn next(&self) -> Option<&Self> {
        self.handle.next().map(|h| h.to_ref())
    }

    // Get mutable reference to previous KLineUnit
    pub fn prev_mut(&self) -> Option<&mut Self> {
        self.handle.prev().map(|h| h.as_mut())
    }

    // Get mutable reference to next KLineUnit
    pub fn next_mut(&self) -> Option<&mut Self> {
        self.handle.next().map(|h| h.as_mut())
    }

    // Get index in the vector
    pub fn index(&self) -> usize {
        self.handle.index()
    }

    fn check(&mut self, autofix: bool) -> Result<(), ChanException> {
        let min_price = self.low.min(self.open).min(self.high).min(self.close);
        let max_price = self.low.max(self.open).max(self.high).max(self.close);

        if self.low > min_price {
            if autofix {
                self.low = min_price;
            } else {
                return Err(ChanException::new(
                    format!("{} low price={} is not min of [low={}, open={}, high={}, close={}]",
                        self.time, self.low, self.low, self.open, self.high, self.close),
                    ErrCode::KlDataInvalid,
                ));
            }
        }

        if self.high < max_price {
            if autofix {
                self.high = max_price;
            } else {
                return Err(ChanException::new(
                    format!("{} high price={} is not max of [low={}, open={}, high={}, close={}]",
                        self.time, self.high, self.low, self.open, self.high, self.close),
                    ErrCode::KlDataInvalid,
                ));
            }
        }
        Ok(())
    }

    pub fn set_metric(&mut self, metric_model_lst: &mut [Box<dyn MetricModel>]) {
        for metric_model in metric_model_lst {
            metric_model.update_kline_unit(self);
        }
    }

    // ... other methods would follow similar patterns
}

// Define a trait for metric models to implement
pub trait MetricModel {
    fn update_kline_unit(&mut self, klu: &mut KLineUnit);
}

// Implement Clone for KLineUnit
impl Clone for KLineUnit {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,  // Handle implements Copy
            kl_type: self.kl_type.clone(),
            time: self.time,
            close: self.close,
            open: self.open,
            high: self.high,
            low: self.low,
            trade_info: self.trade_info.clone(),
            demark: self.demark.clone(),
            sub_kl_list: self.sub_kl_list.clone(),
            sup_kl: self.sup_kl.clone(),
            trend: self.trend.clone(),
            limit_flag: self.limit_flag,
            macd: self.macd.clone(),
            boll: self.boll.clone(),
            rsi: self.rsi,
            kdj: self.kdj.clone(),
        }
    }
}

// Implement PartialEq for KLineUnit
impl PartialEq for KLineUnit {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && 
        self.close == other.close &&
        self.open == other.open &&
        self.high == other.high &&
        self.low == other.low
    }
}