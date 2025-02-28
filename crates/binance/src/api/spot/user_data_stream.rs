use super::prelude::*;
use super::RL_WEIGHT_PER_MINUTE;
use crate::client::Task;

pub const V1_USER_DATA_STREAM: &str = "/api/v1/userDataStream";

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    pub listen_key: String,
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
        /// Create a listenKey.
        ///
        /// Start a new user data stream.
        /// The stream will close after 60 minutes unless a keepalive is sent.
        ///
        /// Weight: 1
        pub fn user_data_stream(&self) -> BinanceResult<Task<ListenKey>> {
            Ok(self
                .rate_limiter
                .task(self.client.post(V1_USER_DATA_STREAM)?.auth_header()?)
                .cost(RL_WEIGHT_PER_MINUTE, 1)
                .send())
        }
    }
}
