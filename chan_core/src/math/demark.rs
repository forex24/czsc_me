use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemarkIndex {
    pub setup_trend: i32,      // -1: down, 0: none, 1: up
    pub setup_idx: i32,        // setup count
    pub countdown_trend: i32,   // -1: down, 0: none, 1: up
    pub countdown_idx: i32,     // countdown count
}

impl Default for DemarkIndex {
    fn default() -> Self {
        Self {
            setup_trend: 0,
            setup_idx: 0,
            countdown_trend: 0,
            countdown_idx: 0,
        }
    }
}

#[derive(Debug)]
pub struct DemarkEngine {
    last_close: Option<f64>,
    setup_count: i32,
    setup_trend: i32,
    countdown_ref_idx: i32,
    countdown_count: i32,
    countdown_trend: i32,
}

impl DemarkEngine {
    pub fn new() -> Self {
        Self {
            last_close: None,
            setup_count: 0,
            setup_trend: 0,
            countdown_ref_idx: 0,
            countdown_count: 0,
            countdown_trend: 0,
        }
    }

    pub fn update(&mut self, close: f64) -> DemarkIndex {
        let mut index = DemarkIndex::default();

        if let Some(last_close) = self.last_close {
            // Setup phase
            if close < last_close {
                if self.setup_trend >= 0 {
                    self.setup_count = 1;
                    self.setup_trend = -1;
                } else {
                    self.setup_count += 1;
                }
            } else if close > last_close {
                if self.setup_trend <= 0 {
                    self.setup_count = 1;
                    self.setup_trend = 1;
                } else {
                    self.setup_count += 1;
                }
            }

            // Reset if setup count reaches 9
            if self.setup_count >= 9 {
                self.setup_count = 0;
                self.countdown_ref_idx = self.setup_trend;
                self.countdown_trend = self.setup_trend;
                self.countdown_count = 0;
            }

            // Countdown phase
            if self.countdown_ref_idx != 0 {
                if (self.countdown_trend > 0 && close <= last_close) ||
                   (self.countdown_trend < 0 && close >= last_close) {
                    self.countdown_count += 1;
                }

                if self.countdown_count >= 13 {
                    self.countdown_count = 0;
                    self.countdown_ref_idx = 0;
                    self.countdown_trend = 0;
                }
            }
        }

        self.last_close = Some(close);

        index.setup_trend = self.setup_trend;
        index.setup_idx = self.setup_count;
        index.countdown_trend = self.countdown_trend;
        index.countdown_idx = self.countdown_count;

        index
    }
} 