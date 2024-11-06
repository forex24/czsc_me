impl<T: LineTrait> ZSList<T> {
    pub fn update(&mut self, bi: Handle<T>, is_sure: bool) {
        if self.free_item_lst.is_empty() && self.try_add_to_end(&bi) {
            self.try_combine();
            return;
        }
        self.add_to_free_lst(bi, is_sure, "normal");
    }

    pub fn try_add_to_end(&mut self, bi: &Handle<T>) -> bool {
        if self.zs_lst.is_empty() {
            false
        } else {
            self.zs_lst.last_mut().unwrap().try_add_to_end(bi)
        }
    }

    pub fn add_zs_from_bi_range(&mut self, seg_bi_lst: &[Handle<T>], seg_dir: BiDir, seg_is_sure: bool) {
        let mut deal_bi_cnt = 0;
        for bi in seg_bi_lst {
            if bi.borrow().dir() == seg_dir {
                continue;
            }
            if deal_bi_cnt < 1 {
                self.add_to_free_lst(bi.clone(), seg_is_sure, "normal");
                deal_bi_cnt += 1;
            } else {
                self.update(bi.clone(), seg_is_sure);
            }
        }
    }

    pub fn try_construct_zs(&self, lst: &[Handle<T>], is_sure: bool, zs_algo: &str) -> Option<ZS<T>> {
        let lst = match zs_algo {
            "normal" => {
                if !self.config.one_bi_zs {
                    if lst.len() == 1 {
                        return None;
                    }
                    &lst[lst.len()-2..]
                } else {
                    lst
                }
            },
            "over_seg" => {
                if lst.len() < 3 {
                    return None;
                }
                let lst = &lst[lst.len()-3..];
                if lst[0].borrow().dir() == lst[0].borrow().parent_seg().unwrap().borrow().dir {
                    return None;
                }
                &lst[1..]
            },
            _ => return None,
        };

        let min_high = lst.iter().map(|item| item.borrow()._high()).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_low = lst.iter().map(|item| item.borrow()._low()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        if min_high > max_low {
            Some(ZS::new(Some(lst), is_sure))
        } else {
            None
        }
    }

    pub fn try_combine(&mut self) {
        if !self.config.need_combine {
            return;
        }
        while self.zs_lst.len() >= 2 {
            let last_idx = self.zs_lst.len() - 1;
            let combine_result = {
                let (first, second) = self.zs_lst.split_at_mut(last_idx);
                first.last_mut().unwrap().combine(
                    &second[0],
                    &self.config.zs_combine_mode
                )
            };
            
            if combine_result {
                self.zs_lst.pop();
            } else {
                break;
            }
        }
    }
}

impl<T: LineTrait> Index<usize> for ZSList<T> {
    type Output = ZS<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.zs_lst[index]
    }
}

impl<T: LineTrait> IndexMut<usize> for ZSList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.zs_lst[index]
    }
}

impl<T: LineTrait> IntoIterator for ZSList<T> {
    type Item = ZS<T>;
    type IntoIter = std::vec::IntoIter<ZS<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.zs_lst.into_iter()
    }
}

impl<T: LineTrait> ZSList<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, ZS<T>> {
        self.zs_lst.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ZS<T>> {
        self.zs_lst.iter_mut()
    }
} 