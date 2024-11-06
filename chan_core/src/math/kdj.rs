#[derive(Debug, Clone)]
pub struct KDJ {
    pub k: f64,
    pub d: f64,
    pub j: f64,
}

#[derive(Debug)]
pub struct KDJModel {
    rsv_period: usize,
    k_period: usize,
    d_period: usize,
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    last_k: f64,
    last_d: f64,
}

impl KDJModel {
    pub fn new(rsv_period: usize, k_period: usize, d_period: usize) -> Self {
        Self {
            rsv_period,
            k_period,
            d_period,
            highs: Vec::with_capacity(rsv_period),
            lows: Vec::with_capacity(rsv_period),
            closes: Vec::with_capacity(rsv_period),
            last_k: 50.0,
            last_d: 50.0,
        }
    }

    pub fn add(&mut self, high: f64, low: f64, close: f64) {
        self.highs.push(high);
        self.lows.push(low);
        self.closes.push(close);

        if self.highs.len() > self.rsv_period {
            self.highs.remove(0);
            self.lows.remove(0);
            self.closes.remove(0);
        }

        let rsv = if self.highs.len() == self.rsv_period {
            let highest = self.highs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let lowest = self.lows.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let close = self.closes.last().unwrap();
            ((close - lowest) / (highest - lowest)) * 100.0
        } else {
            0.0
        };

        self.last_k = if self.k_period == 1 {
            rsv
        } else {
            (self.last_k * (self.k_period - 1) + rsv) / self.k_period
        };

        self.last_d = if self.d_period == 1 {
            self.last_k
        } else {
            (self.last_d * (self.d_period - 1) + self.last_k) / self.d_period
        };
    }
} 