#[derive(Debug, Clone)]
pub struct BollMetric {
    pub up: f64,
    pub mid: f64,
    pub down: f64,
}

#[derive(Debug)]
pub struct BollModel {
    period: usize,
    k: f64,
    prices: Vec<f64>,
}

impl BollModel {
    pub fn new(period: usize, k: f64) -> Self {
        Self {
            period,
            k,
            prices: Vec::with_capacity(period),
        }
    }

    pub fn add(&mut self, price: f64) -> BollMetric {
        self.prices.push(price);
        if self.prices.len() > self.period {
            self.prices.remove(0);
        }

        let mid = self.prices.iter().sum::<f64>() / self.prices.len() as f64;
        
        let variance = self.prices.iter()
            .map(|&x| (x - mid).powi(2))
            .sum::<f64>() / self.prices.len() as f64;
        
        let std_dev = variance.sqrt();
        
        BollMetric {
            up: mid + self.k * std_dev,
            mid,
            down: mid - self.k * std_dev,
        }
    }
} 