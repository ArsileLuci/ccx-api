use serde::de::Deserialize;
use serde::de::Deserializer;
use serde::de::{self};
use serde::ser::Serialize;
use serde::ser::Serializer;

use super::prelude::*;
use super::OrderType;
use super::RlPriorityLevel;
use super::RL_WEIGHT_PER_MINUTE;
use crate::client::Task;
use crate::util::Ask;
use crate::util::Bid;
use crate::util::OrderBook;

pub const API_V3_PING: &str = "/api/v3/ping";
pub const API_V3_TIME: &str = "/api/v3/time";
pub const API_V3_EXCHANGE_INFO: &str = "/api/v3/exchangeInfo";
pub const API_V3_DEPTH: &str = "/api/v3/depth";
pub const API_V3_TRADES: &str = "/api/v3/trades";
pub const API_V3_HISTORICAL_TRADES: &str = "/api/v3/historicalTrades";
pub const API_V3_AGG_TRADES: &str = "/api/v3/aggTrades";
pub const API_V3_KLINES: &str = "/api/v3/klines";
pub const API_V3_AVG_PRICE: &str = "/api/v3/avgPrice";
pub const API_V3_TICKER_24HR: &str = "/api/v3/ticker/24hr";
pub const API_V3_TICKER_PRICE: &str = "/api/v3/ticker/price";
pub const API_V3_TICKER_BOOK_TICKER: &str = "/api/v3/ticker/bookTicker";

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pong {}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInformation {
    pub timezone: Atom,
    pub server_time: u64,
    pub rate_limits: Vec<RateLimit>,
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterval,
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RateLimitType {
    #[serde(rename = "REQUEST_WEIGHT")]
    RequestWeight,
    #[serde(rename = "ORDERS")]
    Orders,
    #[serde(rename = "RAW_REQUESTS")]
    RawRequests,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RateLimitInterval {
    #[serde(rename = "SECOND")]
    Second,
    #[serde(rename = "MINUTE")]
    Minute,
    #[serde(rename = "DAY")]
    Day,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: Atom,
    pub status: SymbolStatus,
    pub base_asset: Atom,
    pub base_asset_precision: u16,
    pub quote_asset: Atom,
    pub quote_precision: u16,
    pub quote_asset_precision: u16,
    pub base_commission_precision: u16,
    pub quote_commission_precision: u16,
    pub order_types: Vec<OrderType>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<Filter>,
    pub permissions: Vec<SymbolPermission>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SymbolStatus {
    #[serde(rename = "PRE_TRADING")]
    PreTrading,
    #[serde(rename = "TRADING")]
    Trading,
    #[serde(rename = "POST_TRADING")]
    PostTrading,
    #[serde(rename = "END_OF_DAY")]
    EndOfDay,
    #[serde(rename = "HALT")]
    Halt,
    #[serde(rename = "AUCTION_MATCH")]
    AuctionMatch,
    #[serde(rename = "BREAK")]
    Break,
}

/// Filters define trading rules on a symbol or an exchange. Filters come in two forms:
/// symbol filters and exchange filters.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "filterType")]
pub enum Filter {
    #[serde(rename = "PRICE_FILTER")]
    Price(PriceFilter),
    #[serde(rename = "PERCENT_PRICE")]
    PercentPrice(PercentPriceFilter),
    #[serde(rename = "PERCENT_PRICE_BY_SIDE")]
    PercentPriceBySide(PercentPriceBySideFilter),
    #[serde(rename = "LOT_SIZE")]
    LotSize(LotSizeFilter),
    #[serde(rename = "MIN_NOTIONAL")]
    MinNotional(MinNotionalFilter),
    #[serde(rename = "NOTIONAL")]
    Notional(NotionalFilter),
    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts(IcebergPartsFilter),
    #[serde(rename = "MARKET_LOT_SIZE")]
    MarketLotSize(MarketLotSizeFilter),
    #[serde(rename = "MAX_NUM_ORDERS")]
    MaxNumOrders(MaxNumOrdersFilter),
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    MaxNumAlgoOrders(MaxNumAlgoOrdersFilter),
    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    MaxNumIcebergOrders(MaxNumIcebergOrdersFilter),
    #[serde(rename = "MAX_POSITION")]
    MaxPosition(MaxPositionFilter),
    #[serde(rename = "TRAILING_DELTA")]
    TrailingDelta(TrailingDeltaFilter),
}

/// The PRICE_FILTER defines the price rules for a symbol. There are 3 parts:
///
/// * `min_price` defines the minimum `price`/`stop_price` allowed;
///   disabled on `min_price` == 0.
/// * `max_price` defines the maximum `price`/`stop_price` allowed;
///   disabled on `max_price` == 0.
/// * `tick_size` defines the intervals that a `price`/`stop_price`
///   can be increased/decreased by; disabled on `tick_size` == 0.
///
/// Any of the above variables can be set to 0, which disables that rule in the price filter.
/// In order to pass the price filter, the following must be true for `price`/`stop_price`
/// of the enabled rules:
///
/// * `price` >= `min_price`
/// * `price` <= `max_price`
/// * (`price` - `min_price`) % `tick_size` == 0
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PriceFilter {
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub tick_size: Decimal,
}

/// The PERCENT_PRICE filter defines valid range for a price based on the average of the previous
/// trades. `avgPriceMins` is the number of minutes the average price is calculated over. 0 means
/// the last price is used.
///
/// In order to pass the percent price, the following must be true for price:
///
/// * `price` <= `weightedAveragePrice` * `multiplierUp`
/// * `price` >= `weightedAveragePrice` * `multiplierDown`
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PercentPriceFilter {
    pub multiplier_up: Decimal,
    pub multiplier_down: Decimal,
    pub avg_price_mins: u64,
}

/// The PERCENT_PRICE_BY_SIDE filter defines the valid range for the price based on the lastPrice
/// of the symbol. There is a different range depending on whether the order is placed
/// on the `BUY` side or the `SELL` side.
///
/// Buy orders will succeed on this filter if:
///
/// * `Order price` <= `bidMultiplierUp` * `lastPrice`
/// * `Order price` >= `bidMultiplierDown` * `lastPrice`
///
/// Sell orders will succeed on this filter if:
///
/// * `Order Price` <= `askMultiplierUp` * `lastPrice`
/// * `Order Price` >= `askMultiplierDown` * `lastPrice`
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct PercentPriceBySideFilter {
    pub bid_multiplier_up: Decimal,
    pub bid_multiplier_down: Decimal,
    pub ask_multiplier_up: Decimal,
    pub ask_multiplier_down: Decimal,
    pub avg_price_mins: u64,
}

/// The LOT_SIZE filter defines the quantity (aka "lots" in auction terms) rules for a symbol.
/// There are 3 parts:
///
/// * `minQty` defines the minimum `quantity`/`icebergQty` allowed.
/// * `maxQty` defines the maximum `quantity`/`icebergQty` allowed.
/// * `stepSize` defines the intervals that a `quantity`/`icebergQty` can be increased/decreased by.
///
/// In order to pass the lot size, the following must be true for `quantity`/`icebergQty`:
///
/// * `quantity` >= `minQty`
/// * `quantity` <= `maxQty`
/// * (`quantity` - `minQty`) % `stepSize` == `0`
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct LotSizeFilter {
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub step_size: Decimal,
}

/// The MIN_NOTIONAL filter defines the minimum notional value allowed for an order on a symbol.
/// An order's notional value is the `price` * `quantity`. If the order is an Algo order
/// (e.g. STOP_LOSS_LIMIT), then the notional value of the `stopPrice` * `quantity` will also be
/// evaluated. If the order is an Iceberg Order, then the notional value of the
/// `price` * `icebergQty` will also be evaluated. `applyToMarket` determines whether or not the
/// MIN_NOTIONAL filter will also be applied to MARKET orders. Since MARKET orders have no `price`,
/// the average price is used over the last `avgPriceMins` minutes. `avgPriceMins` is the number
/// of minutes the average price is calculated over. `0` means the last price is used.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MinNotionalFilter {
    pub min_notional: Decimal,
    pub apply_to_market: bool,
    pub avg_price_mins: u64,
}

/// The NOTIONAL filter defines the acceptable notional range allowed for an order on a symbol.
/// applyMaxToMarket determines whether the maxNotional will be applied to MARKET orders.
///
/// In order to pass this filter, the notional (price * quantity) has to pass the following conditions:
/// price * quantity <= maxNotional
/// price * quantity >= minNotional
///
/// For MARKET orders, the average price used over the last avgPriceMins minutes will be used for calculation.
/// If the avgPriceMins is 0, then the last price will be used.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct NotionalFilter {
    pub min_notional: Decimal,
    pub max_notional: Decimal,
    #[serde(default)]
    pub apply_to_market: bool,
    #[serde(default)]
    pub avg_price_mins: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct IcebergPartsFilter {
    pub limit: u64,
}

/// The MARKET_LOT_SIZE filter defines the quantity (aka "lots" in auction terms) rules for MARKET
/// orders on a symbol. There are 3 parts:
///
/// * `minQty` defines the minimum `quantity`/`icebergQty` allowed.
/// * `maxQty` defines the maximum `quantity`/`icebergQty` allowed.
/// * `stepSize` defines the intervals that a `quantity`/`icebergQty` can be increased/decreased by.
///
/// In order to pass the lot size, the following must be true for `quantity`/`icebergQty`:
///
/// * `quantity` >= `minQty`
/// * `quantity` <= `maxQty`
/// * (`quantity` - `minQty`) % `stepSize` == `0`
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MarketLotSizeFilter {
    pub min_qty: Decimal,
    pub max_qty: Decimal,
    pub step_size: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MaxNumOrdersFilter {
    pub max_num_orders: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MaxNumAlgoOrdersFilter {
    pub max_num_algo_orders: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MaxNumIcebergOrdersFilter {
    pub max_num_iceberg_orders: u64,
}

/// The `MAX_POSITION` filter defines the allowed maximum position an account can have on the
/// base asset of a symbol. An account's position defined as the sum of the account's:
///
/// * free balance of the base asset
/// * locked balance of the base asset
/// * sum of the qty of all open BUY orders
///
/// BUY orders will be rejected if the account's position is greater than the maximum position
/// allowed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct MaxPositionFilter {
    pub max_position: Decimal,
}

/// The `MAX_POSITION` filter defines the allowed maximum position an account can have on the
/// base asset of a symbol. An account's position defined as the sum of the account's:
///
/// * free balance of the base asset
/// * locked balance of the base asset
/// * sum of the qty of all open BUY orders
///
/// BUY orders will be rejected if the account's position is greater than the maximum position
/// allowed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct TrailingDeltaFilter {
    pub min_trailing_above_delta: Decimal,
    pub max_trailing_above_delta: Decimal,
    pub min_trailing_below_delta: Decimal,
    pub max_trailing_below_delta: Decimal,
}

#[derive(Debug, Clone)]
pub enum SymbolPermission {
    Spot,
    Margin,
    Leveraged,
    TradeGroup(u16),
}

impl Serialize for SymbolPermission {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SymbolPermission::Spot => s.serialize_str("SPOT"),
            SymbolPermission::Margin => s.serialize_str("MARGIN"),
            SymbolPermission::Leveraged => s.serialize_str("LEVERAGED"),
            SymbolPermission::TradeGroup(group_num) => {
                let group_num = format!("TRD_GRP_{:0>4}", group_num);
                s.serialize_str(&group_num)
            }
        }
    }
}

impl<'de> Deserialize<'de> for SymbolPermission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match &*s {
            "SPOT" => Ok(Self::Spot),
            "MARGIN" => Ok(Self::Margin),
            "LEVERAGED" => Ok(Self::Leveraged),
            trade_group if trade_group.contains("TRD_GRP") => {
                // Format: TRD_GRP_0001
                let group_num = trade_group.trim_start_matches("TRD_GRP_");
                let group_num = group_num.parse::<u16>().map_err(de::Error::custom)?;
                Ok(Self::TradeGroup(group_num))
            }
            _ => Err(de::Error::custom(format!("invalid type: {}", s))),
        }
    }
}

// FIXME clarify: the documentation is ambiguous; only these values are listed as valid,
//       but below it has a caution about value 0.
//       [https://github.com/binance-exchange/binance-official-api-docs/blob/master/rest-api.md#order-book]
//       If 0 is valid, add it and specify its weight.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OrderBookLimit {
    N5 = 5,
    N10 = 10,
    N20 = 20,
    N50 = 50,
    N100 = 100,
    N500 = 500,
    N1000 = 1000,
    N5000 = 5000,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotOrderBook {
    pub last_update_id: u64,
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: u64,
    pub price: Decimal,
    pub qty: Decimal,
    pub quote_qty: Decimal,
    pub time: u64,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AggTrade {
    /// Aggregate tradeId.
    #[serde(rename = "a")]
    pub id: u64,
    /// Price.
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity.
    #[serde(rename = "q")]
    pub qty: Decimal,
    /// First tradeId.
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    /// Last tradeId.
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    /// Timestamp.
    #[serde(rename = "T")]
    pub time: u64,
    /// Was the buyer the maker?
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    /// Was the trade the best price match?
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

// FIXME serialize as a tuple
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Kline {
    pub open_time: u64,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub close_time: u64,
    pub quote_asset_volume: Decimal,
    pub number_of_trades: u64,
    pub taker_buy_base_asset_volume: Decimal,
    pub taker_buy_quote_asset_volume: Decimal,
    pub ignore: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AvgPrice {
    pub mins: u32,
    pub price: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TickerStats {
    pub symbol: Atom,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub weighted_avg_price: Decimal,
    pub prev_close_price: Decimal,
    pub last_price: Decimal,
    pub last_qty: Decimal,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub open_time: u64,
    pub close_time: u64,
    /// First trade id.
    // FIXME Option<u64> when value is -1
    pub first_id: i64,
    /// Last trade id.
    // FIXME Option<u64> when value is -1
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct PriceTicker {
    pub symbol: Atom,
    pub price: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
    pub symbol: Atom,
    pub bid_price: Decimal,
    pub bid_qty: Decimal,
    pub ask_price: Decimal,
    pub ask_qty: Decimal,
}

impl OrderBookLimit {
    pub fn weight(self) -> u32 {
        use OrderBookLimit as OBL;

        match self {
            OBL::N5 | OBL::N10 | OBL::N20 | OBL::N50 | OBL::N100 => 1,
            OBL::N500 => 5,
            OBL::N1000 => 10,
            OBL::N5000 => 50,
        }
    }

    pub fn as_str(self) -> &'static str {
        use OrderBookLimit as OBL;

        match self {
            OBL::N5 => "5",
            OBL::N10 => "10",
            OBL::N20 => "20",
            OBL::N50 => "50",
            OBL::N100 => "100",
            OBL::N500 => "500",
            OBL::N1000 => "1000",
            OBL::N5000 => "5000",
        }
    }
}

impl From<SpotOrderBook> for OrderBook {
    fn from(book: SpotOrderBook) -> Self {
        OrderBook {
            last_update_id: book.last_update_id,
            bids: book.bids.into(),
            asks: book.asks.into(),
        }
    }
}

#[cfg(feature = "with_network")]
pub use with_network::*;

#[cfg(feature = "with_network")]
mod with_network {
    use super::*;

    impl<S> SpotApi<S>
    where
        S: crate::client::BinanceSigner,
        S: Unpin + 'static,
    {
        /// Test connectivity to the Rest API.
        ///
        /// Weight: 1
        pub fn ping(&self) -> BinanceResult<Task<Pong>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_PING)?)
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// Test connectivity to the Rest API and get the current server time.
        ///
        /// Weight: 1
        pub fn time(&self) -> BinanceResult<Task<ServerTime>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_TIME)?)
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .priority(RlPriorityLevel::Normal as u8)
                .send())
        }

        /// Current exchange trading rules and symbol information.
        ///
        /// Weight: 1
        pub fn exchange_info(&self) -> BinanceResult<Task<ExchangeInformation>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_EXCHANGE_INFO)?)
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// Order book.
        ///
        /// Weight: Adjusted based on the limit:
        ///
        /// Limit | Weight
        /// | ---- | ---- |
        /// 5, 10, 20, 50, 100 | 1
        /// 500 | 5
        /// 1000 | 10
        /// 5000 | 50
        ///
        /// The default `limit` value is `N100`.
        pub fn depth<SM: AsRef<str>>(
            &self,
            symbol: SM,
            limit: impl Into<Option<OrderBookLimit>>,
        ) -> BinanceResult<Task<SpotOrderBook>> {
            let limit: Option<OrderBookLimit> = limit.into();
            let weight = limit.as_ref().map(|f| f.weight()).unwrap_or(1);

            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_DEPTH)?
                        .query_arg("symbol", symbol.as_ref())?
                        .try_query_arg("limit", &limit.map(OrderBookLimit::as_str))?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, weight)
                .send())
        }

        /// Recent trades list.
        ///
        /// Get recent trades.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        /// * `limit` - default 500; max 1000.
        ///
        /// Data Source: Memory
        pub fn trades<SM: AsRef<str>>(
            &self,
            symbol: SM,
            limit: Option<usize>,
        ) -> BinanceResult<Task<Vec<Trade>>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_TRADES)?
                        .query_arg("symbol", symbol.as_ref())?
                        .try_query_arg("limit", &limit)?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// Old Trade Lookup.
        ///
        /// Get older market trades.
        ///
        /// Weight: 5
        ///
        /// Parameters:
        /// * `symbol`
        /// * `from_id` - trade id to fetch from. Default gets most recent trades.
        /// * `limit` - default 500; max 1000.
        ///
        /// Data Source: Database
        pub fn historical_trades<SM: AsRef<str>>(
            &self,
            symbol: SM,
            limit: Option<usize>,
            from_id: Option<u64>,
        ) -> BinanceResult<Task<Vec<Trade>>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_HISTORICAL_TRADES)?
                        .auth_header()?
                        .query_arg("symbol", symbol.as_ref())?
                        .try_query_arg("fromId", &from_id)?
                        .try_query_arg("limit", &limit)?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 5)
                .send())
        }

        /// Compressed/Aggregate trades list.
        ///
        /// Get compressed, aggregate trades. Trades that fill at the time, from the same order,
        /// with the same price will have the quantity aggregated.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        /// * `from_id` - id to get aggregate trades from INCLUSIVE.
        /// * `start_time` - Timestamp in ms to get aggregate trades from INCLUSIVE.
        /// * `end_time` - timestamp in ms to get aggregate trades until INCLUSIVE.
        /// * `limit` - default 500; max 1000.
        ///
        ///
        /// * If both startTime and endTime are sent, time between startTime and endTime
        ///   must be less than 1 hour.
        /// * If fromId, startTime, and endTime are not sent, the most recent aggregate trades
        ///   will be returned.
        ///
        /// Data Source: Database
        pub fn agg_trades<SM: AsRef<str>>(
            &self,
            symbol: SM,
            from_id: Option<u64>,
            start_time: Option<u64>,
            end_time: Option<u64>,
            limit: Option<usize>,
        ) -> BinanceResult<Task<Vec<AggTrade>>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_AGG_TRADES)?
                        .query_arg("symbol", symbol.as_ref())?
                        .try_query_arg("fromId", &from_id)?
                        .try_query_arg("startTime", &start_time)?
                        .try_query_arg("endTime", &end_time)?
                        .try_query_arg("limit", &limit)?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// Kline/Candlestick data.
        ///
        /// Kline/candlestick bars for a symbol.
        /// Klines are uniquely identified by their open time.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        /// * `interval`
        /// * `start_time`
        /// * `end_time`
        /// * `limit` - default 500; max 1000.
        ///
        ///
        /// * If `start_time` and `end_time` are not sent, the most recent klines are returned.
        ///
        /// Data Source: Database
        pub fn klines<SM: AsRef<str>>(
            &self,
            symbol: SM,
            interval: ChartInterval,
            start_time: Option<u64>,
            end_time: Option<u64>,
            limit: Option<usize>,
        ) -> BinanceResult<Task<Vec<Kline>>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_KLINES)?
                        .query_args(&[
                            ("symbol", symbol.as_ref()),
                            ("interval", interval.as_str()),
                        ])?
                        .try_query_arg("startTime", &start_time)?
                        .try_query_arg("endTime", &end_time)?
                        .try_query_arg("limit", &limit)?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// Current average price.
        ///
        /// Current average price for a symbol.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        ///
        /// Data Source: Memory
        pub fn avg_price<SM: AsRef<str>>(&self, symbol: SM) -> BinanceResult<Task<AvgPrice>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_AVG_PRICE)?
                        .query_arg("symbol", symbol.as_ref())?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// 24hr Ticker Price Change Statistics
        ///
        /// 24 hour rolling window price change statistics.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        ///
        /// Data Source: Memory
        pub fn ticker_24hr<SM: AsRef<str>>(&self, symbol: SM) -> BinanceResult<Task<TickerStats>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_TICKER_24HR)?
                        .query_arg("symbol", symbol.as_ref())?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// 24hr Ticker Price Change Statistics
        ///
        /// 24 hour rolling window price change statistics.
        ///
        /// Weight: 40
        ///
        /// Data Source: Memory
        pub fn ticker_24hr_all(&self) -> BinanceResult<Task<Vec<TickerStats>>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_TICKER_24HR)?)
                .cost(RL_WEIGHT_PER_MINUTE, 40)
                .send())
        }

        /// Symbol price ticker.
        ///
        /// Latest price for a symbol.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        ///
        /// Data Source: Memory
        pub fn ticker_price<SM: AsRef<str>>(&self, symbol: SM) -> BinanceResult<Task<PriceTicker>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_TICKER_PRICE)?
                        .query_arg("symbol", symbol.as_ref())?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// All symbol price tickers.
        ///
        /// Latest price for symbols.
        ///
        /// Weight: 2
        ///
        /// Data Source: Memory
        pub fn ticker_price_all(&self) -> BinanceResult<Task<Vec<PriceTicker>>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_TICKER_PRICE)?)
                .cost(RL_WEIGHT_PER_MINUTE, 2)
                .send())
        }

        /// Symbol order book ticker.
        ///
        /// Best price/qty on the order book for a symbol.
        ///
        /// Weight: 1
        ///
        /// Parameters:
        /// * `symbol`
        ///
        /// Data Source: Memory
        pub fn ticker_book<SM: AsRef<str>>(&self, symbol: SM) -> BinanceResult<Task<BookTicker>> {
            Ok(self
                .rate_limiter
                .task(
                    self.client
                        .get(API_V3_TICKER_BOOK_TICKER)?
                        .query_arg("symbol", symbol.as_ref())?,
                )
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }

        /// All symbol order book tickers.
        ///
        /// Best price/qty on the order book for symbols.
        ///
        /// Weight: 2
        ///
        /// Data Source: Memory
        pub fn ticker_book_all(&self) -> BinanceResult<Task<Vec<BookTicker>>> {
            Ok(self
                .rate_limiter
                .task(self.client.get(API_V3_TICKER_BOOK_TICKER)?)
                .cost(RL_WEIGHT_PER_MINUTE, 2)
                .send())
        }
    }
}
