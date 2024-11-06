use std::ops::{Index, IndexMut};
use crate::common::{
    enums::KlineDir,
    chan_exception::{ChanException, ErrCode},
    handle::Handle,
    data_field::DataField,
};
use super::{kline::KLine, kline_unit::KLineUnit};
use crate::impl_handle;

/// K线列表管理器，只负责管理K线的合并和基本操作
#[derive(Debug)]
pub struct KLineList {
    pub kl_type: String,
    pub lst: Vec<KLine>,
}

impl KLineList {
    pub fn new(kl_type: String) -> Self {
        Self {
            kl_type,
            lst: Vec::new(),
        }
    }

    /// Add a single KLineUnit to the list
    pub fn add_single_klu(&mut self, mut klu: KLineUnit) -> Result<(), ChanException> {
        if self.lst.is_empty() {
            self.lst.push(KLine::new(&mut klu, &Box::new(self.lst.clone()), 0, KlineDir::Up));
        } else {
            let dir = self.lst.last_mut().unwrap().try_add(&mut klu, false, None)?;
            
            if dir != KlineDir::Combine {
                self.lst.push(KLine::new(
                    &mut klu,
                    &Box::new(self.lst.clone()),
                    self.lst.len(),
                    dir
                ));
                
                if self.lst.len() >= 3 {
                    let len = self.lst.len();
                    self.lst[len-2].update_fx(false, None);
                }
            }
        }
        Ok(())
    }

    /// Get last KLine
    pub fn last(&self) -> Option<&KLine> {
        self.lst.last()
    }

    /// Get last KLine mutably
    pub fn last_mut(&mut self) -> Option<&mut KLine> {
        self.lst.last_mut()
    }

    /// Get length of KLine list
    pub fn len(&self) -> usize {
        self.lst.len()
    }

    /// Check if KLine list is empty
    pub fn is_empty(&self) -> bool {
        self.lst.is_empty()
    }

    /// Iterator over KLineUnits
    pub fn klu_iter(&self, klc_begin_idx: usize) -> impl Iterator<Item = &KLineUnit> {
        self.lst[klc_begin_idx..].iter().flat_map(|klc| klc.lst.iter())
    }

    /// Deep clone the KLineList
    pub fn deep_clone(&self) -> Self {
        let mut new_obj = Self::new(self.kl_type.clone());
        
        for klc in &self.lst {
            let mut klus_new = Vec::new();
            for klu in &klc.lst {
                let mut new_klu = klu.clone();
                if let Some(pre) = klu.prev() {
                    new_klu.set_pre_klu(pre);
                }
                klus_new.push(new_klu);
            }

            let mut new_klc = KLine::new(
                &mut klus_new[0],
                &Box::new(new_obj.lst.clone()),
                klc.idx,
                klc.dir()
            );
            new_klc.set_fx(klc.fx());
            new_klc.kl_type = klc.kl_type.clone();

            for (idx, klu) in klus_new.iter_mut().enumerate() {
                klu.set_klc(&new_klc);
                if idx != 0 {
                    new_klc.add(klu);
                }
            }

            if let Some(last) = new_obj.lst.last_mut() {
                last.set_next(&new_klc);
                new_klc.set_pre(last);
            }
            new_obj.lst.push(new_klc);
        }

        new_obj
    }
}

// Implement Index trait for array-like access
impl Index<usize> for KLineList {
    type Output = KLine;

    fn index(&self, index: usize) -> &Self::Output {
        &self.lst[index]
    }
}

impl IndexMut<usize> for KLineList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.lst[index]
    }
} 