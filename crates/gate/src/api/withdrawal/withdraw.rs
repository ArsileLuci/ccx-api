use chrono::DateTime;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use serde_with::formats::Flexible;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::TimestampSeconds;
use smart_string::SmartString;

use crate::api::ApiMethod;
use crate::api::ApiVersion;
use crate::api::PrivateRequest;
use crate::api::Request;

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WithdrawalWithdrawRequest {
    /// Client order id, up to 32 length and can only include 0-9, A-Z, a-z, underscore(_), hyphen(-) or dot(.)
    pub withdraw_order_id: Option<SmartString<32>>,
    /// Currency amount
    pub amount: Decimal,
    /// Currency name
    pub currency: SmartString,
    /// Withdrawal address. Required for withdrawals
    pub address: Option<SmartString<66>>,
    /// Additional remarks with regards to the withdrawal
    pub memo: Option<SmartString>,
    /// Name of the chain used in withdrawals
    pub chain: SmartString,
}

impl Request for WithdrawalWithdrawRequest {
    const METHOD: ApiMethod = ApiMethod::Post;
    const VERSION: ApiVersion = ApiVersion::V4;
    type Response = WithdrawalWithdrawResponse;
}

impl PrivateRequest for WithdrawalWithdrawRequest {}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct WithdrawalWithdrawResponse {
    /// Record ID
    pub id: SmartString,
    /// Hash record of the withdrawal
    pub txid: Option<SmartString<64>>,
    /// Client order id, up to 32 length and can only include 0-9, A-Z, a-z, underscore(_), hyphen(-) or dot(.)
    pub withdraw_order_id: Option<SmartString<32>>,
    /// Operation time
    #[serde_as(as = "Option<TimestampSeconds<i64, Flexible>>")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Currency amount
    pub amount: Decimal,
    /// Currency name
    pub currency: SmartString,
    /// Withdrawal address. Required for withdrawals
    pub address: SmartString<66>,
    /// Additional remarks with regards to the withdrawal
    pub memo: Option<SmartString>,
    /// Record status.
    pub status: WithdrawalWithdrawStatus,
    /// Name of the chain used in withdrawals
    pub chain: SmartString,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WithdrawalWithdrawStatus {
    /// done
    Done,
    /// cancelled
    Cancel,
    /// requesting
    Request,
    /// pending manual approval
    Manual,
    /// GateCode operation
    Bcode,
    /// pending confirm after sending
    Extpend,
    /// pending confirm when fail
    Fail,
    /// invalid order
    Invalid,
    /// verifying
    Verify,
    /// processing
    Proces,
    /// pending
    Pend,
    /// required manual approval
    Dmove,
    /// the order is automatically split due to large amount
    Splitpend,
    /// locked. (Missing in docs)
    Locked,
}

impl WithdrawalWithdrawStatus {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            WithdrawalWithdrawStatus::Done
                | WithdrawalWithdrawStatus::Cancel
                | WithdrawalWithdrawStatus::Fail
        )
    }

    pub fn is_pending(&self) -> bool {
        matches!(
            self,
            WithdrawalWithdrawStatus::Request
                | WithdrawalWithdrawStatus::Manual
                | WithdrawalWithdrawStatus::Bcode
                | WithdrawalWithdrawStatus::Extpend
                | WithdrawalWithdrawStatus::Verify
                | WithdrawalWithdrawStatus::Proces
                | WithdrawalWithdrawStatus::Pend
                | WithdrawalWithdrawStatus::Splitpend
        )
    }

    pub fn needs_confirmation(&self) -> bool {
        matches!(self, WithdrawalWithdrawStatus::Manual)
    }
}

#[cfg(feature = "with_network")]
mod with_network {
    use super::*;
    use crate::api::withdrawal::WithdrawalApi;
    use crate::client::rest::RequestError;
    use crate::client::signer::GateSigner;

    impl<S: GateSigner> WithdrawalApi<S> {
        /// # Withdraw
        ///
        /// Withdraw
        ///
        /// ‼️ Be aware that Client order id does not guarantee the uniqueness of the order id on the Gate API side.
        ///
        /// ## Parameters
        ///
        /// * `withdraw_order_id` - Client order id, up to 32 length and can only include 0-9, A-Z, a-z, underscore(_),
        ///   hyphen(-) or dot(.)
        /// * `amount` - Currency amount
        /// * `currency` - Currency name
        /// * `address` - Withdrawal address. Required for withdrawals
        /// * `memo` - Additional remarks with regards to the withdrawal
        /// * `chain` - Name of the chain used in withdrawals
        pub async fn withdraw(
            &self,
            request: &WithdrawalWithdrawRequest,
        ) -> Result<<WithdrawalWithdrawRequest as Request>::Response, RequestError> {
            self.0.signed_request("/withdrawals", request).await
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_example_from_docs() {
        let json = r#"{
            "id": "210496",
            "timestamp": "1542000000",
            "withdraw_order_id": "order_123456",
            "currency": "USDT",
            "address": "1HkxtBAMrA3tP5ENnYY2CZortjZvFDH5Cs",
            "txid": "128988928203223323290",
            "amount": "222.61",
            "memo": "",
            "status": "DONE",
            "chain": "TRX"
        }"#;
        let res: WithdrawalWithdrawResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            res,
            WithdrawalWithdrawResponse {
                id: "210496".into(),
                timestamp: DateTime::from_timestamp(1542000000, 0),
                withdraw_order_id: Some("order_123456".into()),
                currency: "USDT".into(),
                address: "1HkxtBAMrA3tP5ENnYY2CZortjZvFDH5Cs".into(),
                txid: Some("128988928203223323290".into()),
                amount: dec!(222.61),
                memo: Some("".into()),
                status: WithdrawalWithdrawStatus::Done,
                chain: "TRX".into(),
            }
        );
    }

    #[test]
    fn test_example_real_response() {
        let json = r#"{"id":"w50000000","currency":"USDT","amount":"2.63","address":"Txxx","memo":null,"status":"REQUEST","chain":"TRX","withdraw_order_id":"47eaed6f32f24cb7a765fef1966e775b"}"#;
        let res: WithdrawalWithdrawResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            res,
            WithdrawalWithdrawResponse {
                id: "w50000000".into(),
                timestamp: None,
                withdraw_order_id: Some("47eaed6f32f24cb7a765fef1966e775b".into()),
                currency: "USDT".into(),
                address: "Txxx".into(),
                txid: None,
                amount: dec!(2.63),
                memo: None,
                status: WithdrawalWithdrawStatus::Request,
                chain: "TRX".into(),
            }
        );
    }
}
