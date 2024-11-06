#[derive(Debug)]
pub struct RSI {
    period: usize,
    last_price: Option<f64>,
    gains: Vec<f64>,
    losses: Vec<f64>,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            last_price: None,
            gains: Vec::with_capacity(period),
            losses: Vec::with_capacity(period),
        }
    }

    pub fn add(&mut self, price: f64) -> Option<f64> {
        if let Some(last_price) = self.last_price {
            let change = price - last_price;
            
            if change >= 0.0 {
                self.gains.push(change);
                self.losses.push(0.0);
            } else {
                self.gains.push(0.0);
                self.losses.push(-change);
            }

            if self.gains.len() > self.period {
                self.gains.remove(0);
                self.losses.remove(0);
            }

            if self.gains.len() == self.period {
                let avg_gain = self.gains.iter().sum::<f64>() / self.period as f64;
                let avg_loss = self.losses.iter().sum::<f64>() / self.period as f64;

                if avg_loss == 0.0 {
                    Some(100.0)
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(100.0 - (100.0 / (1.0 + rs)))
                }
            } else {
                None
            }
        } else {
            None
        }.map(|rsi| rsi.max(0.0).min(100.0))
    }
} 