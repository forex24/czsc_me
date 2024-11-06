#[derive(Debug, Clone)]
pub struct MACDItem {
    pub dif: f64,
    pub dea: f64,
    pub macd: f64,
}

#[derive(Debug)]
pub struct MACD {
    short_ema: f64,
    long_ema: f64,
    dea: f64,
    short_period: u32,
    long_period: u32,
    dea_period: u32,
    count: u32,
}

impl MACD {
    pub fn new(short_period: u32, long_period: u32, dea_period: u32) -> Self {
        Self {
            short_ema: 0.0,
            long_ema: 0.0,
            dea: 0.0,
            short_period,
            long_period,
            dea_period,
            count: 0,
        }
    }

    pub fn add(&mut self, price: f64) -> MACDItem {
        self.count += 1;
        
        // Calculate EMAs
        if self.count == 1 {
            self.short_ema = price;
            self.long_ema = price;
        } else {
            self.short_ema = (2.0 * price + (self.short_period as f64 - 1.0) * self.short_ema) 
                / (self.short_period as f64 + 1.0);
            self.long_ema = (2.0 * price + (self.long_period as f64 - 1.0) * self.long_ema) 
                / (self.long_period as f64 + 1.0);
        }

        let dif = self.short_ema - self.long_ema;
        
        // Calculate DEA
        if self.count == 1 {
            self.dea = dif;
        } else {
            self.dea = (2.0 * dif + (self.dea_period as f64 - 1.0) * self.dea) 
                / (self.dea_period as f64 + 1.0);
        }

        MACDItem {
            dif,
            dea: self.dea,
            macd: 2.0 * (dif - self.dea),
        }
    }
} 