use std::fmt;
use strum_macros::{Display, EnumString};

/// Error codes for the Chan system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString)]
#[repr(i32)]
pub enum ErrCode {
    // Chan errors (0-99)
    #[strum(serialize = "_CHAN_ERR_BEGIN")]
    ChanErrBegin = 0,
    #[strum(serialize = "COMMON_ERROR")]
    CommonError = 1,
    #[strum(serialize = "SRC_DATA_NOT_FOUND")]
    SrcDataNotFound = 3,
    #[strum(serialize = "SRC_DATA_TYPE_ERR")]
    SrcDataTypeErr = 4,
    #[strum(serialize = "PARA_ERROR")]
    ParaError = 5,
    #[strum(serialize = "EXTRA_KLU_ERR")]
    ExtraKluErr = 6,
    #[strum(serialize = "SEG_END_VALUE_ERR")]
    SegEndValueErr = 7,
    #[strum(serialize = "SEG_EIGEN_ERR")]
    SegEigenErr = 8,
    #[strum(serialize = "BI_ERR")]
    BiErr = 9,
    #[strum(serialize = "COMBINER_ERR")]
    CombinerErr = 10,
    #[strum(serialize = "PLOT_ERR")]
    PlotErr = 11,
    #[strum(serialize = "MODEL_ERROR")]
    ModelError = 12,
    #[strum(serialize = "SEG_LEN_ERR")]
    SegLenErr = 13,
    #[strum(serialize = "ENV_CONF_ERR")]
    EnvConfErr = 14,
    #[strum(serialize = "UNKNOWN_DB_TYPE")]
    UnknownDbType = 15,
    #[strum(serialize = "FEATURE_ERROR")]
    FeatureError = 16,
    #[strum(serialize = "CONFIG_ERROR")]
    ConfigError = 17,
    #[strum(serialize = "SRC_DATA_FORMAT_ERROR")]
    SrcDataFormatError = 18,
    #[strum(serialize = "_CHAN_ERR_END")]
    ChanErrEnd = 99,

    // Trade errors (100-199)
    #[strum(serialize = "_TRADE_ERR_BEGIN")]
    TradeErrBegin = 100,
    #[strum(serialize = "SIGNAL_EXISTED")]
    SignalExisted = 101,
    #[strum(serialize = "RECORD_NOT_EXIST")]
    RecordNotExist = 102,
    #[strum(serialize = "RECORD_ALREADY_OPENED")]
    RecordAlreadyOpened = 103,
    #[strum(serialize = "QUOTA_NOT_ENOUGH")]
    QuotaNotEnough = 104,
    #[strum(serialize = "RECORD_NOT_OPENED")]
    RecordNotOpened = 105,
    #[strum(serialize = "TRADE_UNLOCK_FAIL")]
    TradeUnlockFail = 106,
    #[strum(serialize = "PLACE_ORDER_FAIL")]
    PlaceOrderFail = 107,
    #[strum(serialize = "LIST_ORDER_FAIL")]
    ListOrderFail = 108,
    #[strum(serialize = "CANDEL_ORDER_FAIL")]
    CandelOrderFail = 109,
    #[strum(serialize = "GET_FUTU_PRICE_FAIL")]
    GetFutuPriceFail = 110,
    #[strum(serialize = "GET_FUTU_LOT_SIZE_FAIL")]
    GetFutuLotSizeFail = 111,
    #[strum(serialize = "OPEN_RECORD_NOT_WATCHING")]
    OpenRecordNotWatching = 112,
    #[strum(serialize = "GET_HOLDING_QTY_FAIL")]
    GetHoldingQtyFail = 113,
    #[strum(serialize = "RECORD_CLOSED")]
    RecordClosed = 114,
    #[strum(serialize = "REQUEST_TRADING_DAYS_FAIL")]
    RequestTradingDaysFail = 115,
    #[strum(serialize = "COVER_ORDER_ID_NOT_UNIQUE")]
    CoverOrderIdNotUnique = 116,
    #[strum(serialize = "SIGNAL_TRADED")]
    SignalTraded = 117,
    #[strum(serialize = "_TRADE_ERR_END")]
    TradeErrEnd = 199,

    // KL data errors (200-299)
    #[strum(serialize = "_KL_ERR_BEGIN")]
    KlErrBegin = 200,
    #[strum(serialize = "PRICE_BELOW_ZERO")]
    PriceBelowZero = 201,
    #[strum(serialize = "KL_DATA_NOT_ALIGN")]
    KlDataNotAlign = 202,
    #[strum(serialize = "KL_DATA_INVALID")]
    KlDataInvalid = 203,
    #[strum(serialize = "KL_TIME_INCONSISTENT")]
    KlTimeInconsistent = 204,
    #[strum(serialize = "TRADEINFO_TOO_MUCH_ZERO")]
    TradeinfoTooMuchZero = 205,
    #[strum(serialize = "KL_NOT_MONOTONOUS")]
    KlNotMonotonous = 206,
    #[strum(serialize = "SNAPSHOT_ERR")]
    SnapshotErr = 207,
    #[strum(serialize = "SUSPENSION")]
    Suspension = 208, // 疑似停牌
    #[strum(serialize = "STOCK_IPO_TOO_LATE")]
    StockIpoTooLate = 209,
    #[strum(serialize = "NO_DATA")]
    NoData = 210,
    #[strum(serialize = "STOCK_NOT_ACTIVE")]
    StockNotActive = 211,
    #[strum(serialize = "STOCK_PRICE_NOT_ACTIVE")]
    StockPriceNotActive = 212,
    #[strum(serialize = "_KL_ERR_END")]
    KlErrEnd = 299,
}

impl ErrCode {
    pub fn is_kldata_err(&self) -> bool {
        let code = *self as i32;
        code > Self::KlErrBegin as i32 && code < Self::KlErrEnd as i32
    }

    pub fn is_chan_err(&self) -> bool {
        let code = *self as i32;
        code > Self::ChanErrBegin as i32 && code < Self::ChanErrEnd as i32
    }
}

#[derive(Debug)]
pub struct ChanError {
    pub errcode: ErrCode,
    pub msg: String,
}

impl ChanError {
    pub fn new(message: impl Into<String>, code: ErrCode) -> Self {
        Self {
            errcode: code,
            msg: message.into(),
        }
    }

    pub fn is_kldata_err(&self) -> bool {
        self.errcode.is_kldata_err()
    }

    pub fn is_chan_err(&self) -> bool {
        self.errcode.is_chan_err()
    }
}

impl std::error::Error for ChanError {}

impl fmt::Display for ChanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.errcode, self.msg)
    }
}
