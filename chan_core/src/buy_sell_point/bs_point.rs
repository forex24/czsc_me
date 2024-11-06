use std::collections::HashMap;
use strum_macros::{Display, EnumString};
use crate::chan_model::features::Features;
use crate::common::handle::Handle;
use crate::kline::kline_unit::KLineUnit;
use crate::traits::line_trait::LineTrait;

#[derive(Debug, Clone, Display, EnumString, PartialEq)]
pub enum BspType {
    #[strum(serialize = "BS1")]
    BS1,
    #[strum(serialize = "BS2")]
    BS2,
    #[strum(serialize = "BS3")]
    BS3,
    // ... 其他买卖点类型
}

pub struct BSPoint<T: LineTrait> {
    /// The bi/seg this point belongs to
    pub bi: Handle<T>,
    
    /// The KLineUnit at the end of bi/seg
    pub klu: Handle<KLineUnit>,
    
    /// Whether this is a buy point
    pub is_buy: bool,
    
    /// Types of this buy/sell point
    pub bs_type: Vec<BspType>,
    
    /// Related BS point
    pub relate_bsp1: Option<Handle<BSPoint<T>>>,
    
    /// Features of this point
    pub features: Features,
    
    /// Whether this is a segment buy/sell point
    pub is_segbsp: bool,
}

impl<T: LineTrait> BSPoint<T> {
    pub fn new(
        bi: Handle<T>,
        is_buy: bool,
        bs_type: BspType,
        relate_bsp1: Option<Handle<BSPoint<T>>>,
        feature_dict: Option<HashMap<String, f64>>,
    ) -> Self {
        let klu = bi.borrow().get_end_klu();
        let features = Features::new(feature_dict);
        
        let mut bsp = Self {
            bi,
            klu,
            is_buy,
            bs_type: vec![bs_type],
            relate_bsp1,
            features,
            is_segbsp: false,
        };
        
        // Set the bsp reference in bi
        bsp.bi.borrow_mut().set_bsp(Handle::new(bsp.clone()));
        
        // Initialize common features
        bsp.init_common_feature();
        
        bsp
    }

    pub fn add_type(&mut self, bs_type: BspType) {
        self.bs_type.push(bs_type);
    }

    pub fn type_to_string(&self) -> String {
        self.bs_type
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn add_another_bsp_prop(
        &mut self,
        bs_type: BspType,
        relate_bsp1: Option<Handle<BSPoint<T>>>,
    ) {
        self.add_type(bs_type);
        
        if self.relate_bsp1.is_none() {
            self.relate_bsp1 = relate_bsp1;
        } else if let Some(new_bsp1) = relate_bsp1 {
            assert_eq!(
                self.relate_bsp1.as_ref().unwrap().borrow().klu.borrow().idx,
                new_bsp1.borrow().klu.borrow().idx
            );
        }
    }

    pub fn add_feat<K>(&mut self, inp1: K, inp2: Option<f64>)
    where
        K: Into<Features> + std::fmt::Debug,
    {
        self.features.add_feat(inp1, inp2);
    }

    pub fn init_common_feature(&mut self) {
        // Initialize features that apply to all buy/sell points
        let mut common_features = HashMap::new();
        common_features.insert(
            "bsp_bi_amp".to_string(),
            self.bi.borrow().amp(),
        );
        
        self.add_feat(common_features, None);
    }
}

impl<T: LineTrait> Clone for BSPoint<T> {
    fn clone(&self) -> Self {
        Self {
            bi: self.bi.clone(),
            klu: self.klu.clone(),
            is_buy: self.is_buy,
            bs_type: self.bs_type.clone(),
            relate_bsp1: self.relate_bsp1.clone(),
            features: self.features.clone(),
            is_segbsp: self.is_segbsp,
        }
    }
} 