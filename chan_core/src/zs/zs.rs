impl<T: LineTrait> ZS<T> {
    pub fn combine(&mut self, zs2: &ZS<T>, combine_mode: &str) -> bool {
        if zs2.is_one_bi_zs() {
            return false;
        }

        if let (Some(begin1), Some(begin2)) = (&self.begin_bi, &zs2.begin_bi) {
            if begin1.borrow().seg_idx() != begin2.borrow().seg_idx() {
                return false;
            }
        }

        match combine_mode {
            "zs" => {
                if !has_overlap(
                    self.low.unwrap(),
                    self.high.unwrap(),
                    zs2.low.unwrap(),
                    zs2.high.unwrap(),
                    true,
                ) {
                    return false;
                }
                self.do_combine(zs2);
                true
            }
            "peak" => {
                if has_overlap(
                    self.peak_low,
                    self.peak_high,
                    zs2.peak_low,
                    zs2.peak_high,
                    true,
                ) {
                    self.do_combine(zs2);
                    true
                } else {
                    false
                }
            }
            _ => Err(ChanError::new(
                format!("{} is unsupport zs combine mode", combine_mode),
                ErrCode::ParaError,
            ))?,
        }
    }

    pub fn do_combine(&mut self, zs2: &ZS<T>) {
        if self.sub_zs_lst.is_empty() {
            self.sub_zs_lst.push(self.make_copy());
        }
        self.sub_zs_lst.push(zs2.make_copy());

        self.low = Some(self.low.unwrap().min(zs2.low.unwrap()));
        self.high = Some(self.high.unwrap().max(zs2.high.unwrap()));
        self.peak_low = self.peak_low.min(zs2.peak_low);
        self.peak_high = self.peak_high.max(zs2.peak_high);
        self.end = zs2.end.clone();
        self.bi_out = zs2.bi_out.clone();
        self.end_bi = zs2.end_bi.clone();
    }

    pub fn try_add_to_end(&mut self, item: &Handle<T>) -> bool {
        if !self.in_range(item) {
            return false;
        }
        if self.is_one_bi_zs() {
            if let Some(begin_bi) = &self.begin_bi {
                self.update_zs_range(&[begin_bi.clone(), item.clone()]);
            }
        }
        self.update_zs_end(item);
        true
    }

    pub fn in_range(&self, item: &Handle<T>) -> bool {
        has_overlap(
            self.low.unwrap(),
            self.high.unwrap(),
            item.borrow()._low(),
            item.borrow()._high(),
            true,
        )
    }

    pub fn is_inside(&self, seg: &Handle<Seg>) -> bool {
        if let (Some(begin_bi), Some(start_bi), Some(end_bi)) =
            (&self.begin_bi, &seg.borrow().start_bi, &seg.borrow().end_bi)
        {
            start_bi.borrow().idx() <= begin_bi.borrow().idx()
                && begin_bi.borrow().idx() <= end_bi.borrow().idx()
        } else {
            false
        }
    }

    pub fn is_divergence(
        &self,
        config: &PointConfig,
        out_bi: Option<&Handle<T>>,
    ) -> (bool, Option<f64>) {
        if !self.end_bi_break(out_bi) {
            return (false, None);
        }

        let in_metric = self
            .get_bi_in()
            .borrow()
            .cal_macd_metric(&config.macd_algo, false);
        let out_metric = if let Some(out) = out_bi {
            out.borrow().cal_macd_metric(&config.macd_algo, true)
        } else {
            self.get_bi_out()
                .borrow()
                .cal_macd_metric(&config.macd_algo, true)
        };

        let ratio = out_metric / in_metric;
        if config.divergence_rate > 100.0 {
            (true, Some(ratio))
        } else {
            (
                out_metric <= config.divergence_rate * in_metric,
                Some(ratio),
            )
        }
    }

    pub fn make_copy(&self) -> Self {
        Self {
            is_sure: self.is_sure,
            sub_zs_lst: self.sub_zs_lst.clone(),
            begin: self.begin.clone(),
            begin_bi: self.begin_bi.clone(),
            low: self.low,
            high: self.high,
            mid: self.mid,
            end: self.end.clone(),
            end_bi: self.end_bi.clone(),
            peak_high: self.peak_high,
            peak_low: self.peak_low,
            bi_in: self.bi_in.clone(),
            bi_out: self.bi_out.clone(),
            bi_lst: self.bi_lst.clone(),
        }
    }

    pub fn end_bi_break(&self, end_bi: Option<&Handle<T>>) -> bool {
        let end_bi = if let Some(bi) = end_bi {
            bi
        } else {
            self.get_bi_out()
        };

        let end_bi = end_bi.borrow();
        (end_bi.is_down() && end_bi._low() < self.low.unwrap())
            || (end_bi.is_up() && end_bi._high() > self.high.unwrap())
    }

    pub fn out_bi_is_peak(&self, end_bi_idx: usize) -> (bool, Option<f64>) {
        if self.bi_lst.is_empty() || self.bi_out.is_none() {
            return (false, None);
        }

        let bi_out = self.bi_out.as_ref().unwrap();
        let mut peak_rate = f64::INFINITY;

        for bi in &self.bi_lst {
            if bi.borrow().idx() > end_bi_idx {
                break;
            }

            let bi = bi.borrow();
            let bi_out = bi_out.borrow();

            if (bi_out.is_down() && bi._low() < bi_out._low())
                || (bi_out.is_up() && bi._high() > bi_out._high())
            {
                return (false, None);
            }

            let r = (bi.get_end_val() - bi_out.get_end_val()).abs() / bi_out.get_end_val();
            if r < peak_rate {
                peak_rate = r;
            }
        }

        (true, Some(peak_rate))
    }

    pub fn get_bi_in(&self) -> &Handle<T> {
        self.bi_in.as_ref().expect("bi_in should not be None")
    }

    pub fn get_bi_out(&self) -> &Handle<T> {
        self.bi_out.as_ref().expect("bi_out should not be None")
    }

    pub fn set_bi_in(&mut self, bi: Handle<T>) {
        self.bi_in = Some(bi);
    }

    pub fn set_bi_out(&mut self, bi: Handle<T>) {
        self.bi_out = Some(bi);
    }

    pub fn set_bi_lst(&mut self, bi_lst: Vec<Handle<T>>) {
        self.bi_lst = bi_lst;
    }

    pub fn init_from_zs(&mut self, zs: &ZS<T>) {
        self.begin = zs.begin.clone();
        self.end = zs.end.clone();
        self.low = zs.low;
        self.high = zs.high;
        self.peak_high = zs.peak_high;
        self.peak_low = zs.peak_low;
        self.begin_bi = zs.begin_bi.clone();
        self.end_bi = zs.end_bi.clone();
        self.bi_in = zs.bi_in.clone();
        self.bi_out = zs.bi_out.clone();
    }
}

impl<T: LineTrait> std::fmt::Display for ZS<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let begin_idx = self.begin_bi.as_ref().map_or(0, |bi| bi.borrow().idx());
        let end_idx = self.end_bi.as_ref().map_or(0, |bi| bi.borrow().idx());

        let base_str = format!("{}->{}", begin_idx, end_idx);

        if self.sub_zs_lst.is_empty() {
            write!(f, "{}", base_str)
        } else {
            let sub_str: Vec<String> = self.sub_zs_lst.iter().map(|zs| zs.to_string()).collect();
            write!(f, "{}({})", base_str, sub_str.join(","))
        }
    }
}
