pub mod create;
pub mod get;
pub mod list;

use chrono::DateTime;
use chrono::Utc;
use create::CreateOrderRequest;
use displaydoc::Display;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use serde_with::formats::Flexible;
use serde_with::serde_as;
use serde_with::TimestampMilliSeconds;
use smart_string::SmartString;

#[cfg(feature = "with_network")]
mod with_network {
    use create::CreateOrderRequest;
    use get::GetOrderParams;
    use list::ListOrdersRequest;

    use super::*;
    use crate::api::spot::SpotApi;
    use crate::client::rest::RequestError;
    use crate::client::signer::GateSigner;

    impl<S: GateSigner> SpotApi<S> {
        /// Create an order
        ///
        /// # Endpoint
        /// `POST /spot/orders`
        pub async fn create_order(
            &self,
            request: &CreateOrderRequest,
        ) -> Result<Order, RequestError> {
            self.0.signed_request("/spot/orders", request).await
        }

        /// List orders
        ///
        /// # Endpoint
        /// `GET /spot/orders`
        ///
        /// # Description
        /// Spot, portfolio and margin orders are returned by default.
        /// If cross margin orders are needed, `account` must be set to `cross_margin`
        pub async fn list_orders(
            &self,
            request: &ListOrdersRequest,
        ) -> Result<Vec<Order>, RequestError> {
            self.0.signed_request("/spot/orders", request).await
        }

        /// Get a single order
        ///
        /// # Endpoint
        /// `GET /spot/orders/{order_id}`
        ///
        /// # Description
        /// This endpoint retrieves detailed information about a specific order.
        pub async fn get_order(
            &self,
            id: &str,
            params: &GetOrderParams,
        ) -> Result<Order, RequestError> {
            let path = format!("/spot/orders/{id}");
            self.0.signed_request(&path, params).await
        }
    }
}

/// Represents the details of a spot order.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Order {
    /// Order fields derived from order creation request
    #[serde(flatten)]
    pub request: CreateOrderRequest,

    /// Order ID (read-only).
    pub id: SmartString<15>,

    /// The custom data that the user remarked when amending the order (read-only).
    pub amend_text: Option<SmartString>,

    /// Creation time of the order (read-only).
    #[serde(rename = "create_time_ms")]
    #[serde_as(as = "TimestampMilliSeconds<i64, Flexible>")]
    pub create_time: DateTime<Utc>,

    /// Last modification time of the order (read-only).
    #[serde(rename = "update_time_ms")]
    #[serde_as(as = "TimestampMilliSeconds<i64, Flexible>")]
    pub update_time: DateTime<Utc>,

    /// Order status (read-only).
    pub status: OrderStatus,

    /// Amount left to fill (read-only).
    pub left: Option<Decimal>,

    /// Amount traded to fill (read-only).
    pub filled_amount: Option<Decimal>,

    /// Total filled in base currency (read-only).
    pub fill_price: Option<Decimal>,

    /// Total filled in quote currency (read-only).
    pub filled_total: Option<Decimal>,

    /// Average fill price (read-only).
    pub avg_deal_price: Option<Decimal>,

    /// Fee deducted (read-only).
    pub fee: Option<Decimal>,

    /// Fee currency unit (read-only).
    pub fee_currency: Option<SmartString<8>>,

    /// Points used to deduct fee (read-only).
    pub point_fee: Option<Decimal>,

    /// GT used to deduct fee (read-only).
    pub gt_fee: Option<Decimal>,

    /// GT used to deduct maker fee (read-only).
    pub gt_maker_fee: Option<Decimal>,

    /// GT used to deduct taker fee (read-only).
    pub gt_taker_fee: Option<Decimal>,

    /// Whether GT fee discount is used (read-only).
    pub gt_discount: Option<bool>,

    /// Rebated fee (read-only).
    pub rebated_fee: Option<Decimal>,

    /// Rebated fee currency unit (read-only).
    pub rebated_fee_currency: Option<SmartString<8>>,

    /// Orders between users in the same stp_id group are not allowed to be self-traded (read-only).
    pub stp_id: Option<i64>,

    /// Order completion statuses (read-only).
    pub finish_as: FinishAs,
}

/// Represents the status of an order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(PartialEq))]
pub enum OrderStatus {
    /// Order is open and waiting to be filled.
    Open,

    /// Order is closed and fully filled.
    Closed,

    /// Order is cancelled.
    Cancelled,
}

/// Represents the possible completion statuses of an order.
#[derive(Debug, Clone, Copy, Deserialize, Display)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum FinishAs {
    /// Awaiting processing.
    Open,

    /// Fully filled.
    Filled,

    /// Cancelled by user.
    Cancelled,

    /// Cancelled due to liquidation.
    LiquidateCancelled,

    /// Cancelled due to insufficient market depth.
    DepthNotEnough,

    /// Cancelled due to insufficient counterparty.
    TraderNotEnough,

    /// Order quantity too small.
    Small,

    /// Not immediately filled because TIF is set to IOC.
    Ioc,

    /// Not met the order strategy because TIF is set to POC.
    Poc,

    /// Not fully filled immediately because TIF is set to FOK.
    Fok,

    /// Cancelled due to self-trade prevention.
    Stp,

    /// Unknown.
    Unknown,
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use serde_json;
    use similar_asserts::assert_eq;

    use super::super::order::*;

    #[test]
    fn deserialize_order() {
        // Create an example instance of Order
        let expected = Order {
            id: "1852454420".into(),
            request: CreateOrderRequest {
                text: Some("t-abc123".into()),
                currency_pair: "BTC_USDT".into(),
                order_type: Some(create::OrderType::Limit),
                account: Some(create::AccountType::Spot),
                side: create::OrderSide::Buy,
                amount: dec!(0.001),
                price: Some(dec!(65000)),
                time_in_force: Some(create::TimeInForce::GoodTillCancelled),
                iceberg: Some(dec!(0)),
                auto_borrow: None,
                auto_repay: None,
                stp_action: None,
                action_mode: None,
            },
            amend_text: Some("-".into()),
            create_time: DateTime::from_timestamp_millis(1710488334073).unwrap(),
            update_time: DateTime::from_timestamp_millis(1710488334074).unwrap(),
            status: OrderStatus::Closed,
            left: Some(dec!(0)),
            filled_amount: Some(dec!(0.001)),
            fill_price: Some(dec!(63.4693)),
            filled_total: Some(dec!(63.4693)),
            avg_deal_price: Some(dec!(63.4693)),
            fee: Some(dec!(0.00000022)),
            fee_currency: Some("BTC".into()),
            point_fee: Some(dec!(0)),
            gt_fee: Some(dec!(0)),
            gt_maker_fee: Some(dec!(0)),
            gt_taker_fee: Some(dec!(0)),
            gt_discount: Some(false),
            rebated_fee: Some(dec!(0)),
            rebated_fee_currency: Some("USDT".into()),
            stp_id: None,
            finish_as: FinishAs::Filled,
        };

        let json = r#"{
  "id": "1852454420",
  "text": "t-abc123",
  "amend_text": "-",
  "create_time": "1710488334",
  "update_time": "1710488334",
  "create_time_ms": 1710488334073,
  "update_time_ms": 1710488334074,
  "status": "closed",
  "currency_pair": "BTC_USDT",
  "type": "limit",
  "account": "spot",
  "side": "buy",
  "amount": "0.001",
  "price": "65000",
  "time_in_force": "gtc",
  "iceberg": "0",
  "left": "0",
  "filled_amount": "0.001",
  "fill_price": "63.4693",
  "filled_total": "63.4693",
  "avg_deal_price": "63.4693",
  "fee": "0.00000022",
  "fee_currency": "BTC",
  "point_fee": "0",
  "gt_fee": "0",
  "gt_maker_fee": "0",
  "gt_taker_fee": "0",
  "gt_discount": false,
  "rebated_fee": "0",
  "rebated_fee_currency": "USDT",
  "finish_as": "filled"
}"#;

        // Assert that the original and deserialized orders are the same
        assert_eq!(expected, serde_json::from_str(json).unwrap());
    }
}
