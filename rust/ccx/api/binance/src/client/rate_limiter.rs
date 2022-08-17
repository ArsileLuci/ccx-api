use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix::clock::sleep;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::lock::Mutex;
use futures::prelude::*;
use futures::task::Context;
use futures::task::Poll;

use super::BinanceSigner;
use super::RequestBuilder;
use crate::BinanceResult;
use crate::LibError;

type BucketName = Cow<'static, str>;
type TaskCosts = HashMap<BucketName, u32>;
type TaskMessageResult = BinanceResult<()>;

struct TaskMessage {
    costs: TaskCosts,
    task_tx: oneshot::Sender<TaskMessageResult>,
}

#[derive(Default)]
pub(crate) struct RateLimiterBuilder {
    buckets: HashMap<BucketName, RateLimiterBucket>,
}

impl RateLimiterBuilder {
    pub fn bucket(mut self, key: impl Into<BucketName>, bucket: RateLimiterBucket) -> Self {
        match self.buckets.entry(key.into()) {
            Entry::Occupied(mut e) => *e.get_mut() = bucket,
            Entry::Vacant(e) => {
                e.insert(bucket);
            }
        }
        self
    }

    pub fn start(self) -> RateLimiter {
        let (queue_tx, queue_rx) = mpsc::unbounded::<TaskMessage>();
        let rate_limiter = RateLimiter {
            buckets: Arc::new(
                self.buckets
                    .into_iter()
                    .map(|(k, v)| (k, Mutex::new(v.into())))
                    .collect(),
            ),
            queue_tx,
            // queue: Arc::new(Mutex::new(Vec::new())),
        };
        rate_limiter.recv(queue_rx);
        rate_limiter
    }
}

#[derive(Clone)]
pub(crate) struct RateLimiter {
    buckets: Arc<HashMap<BucketName, Mutex<RateLimiterBucket>>>,
    queue_tx: mpsc::UnboundedSender<TaskMessage>,
    // queue: Arc<Mutex<Vec<TaskMessage>>>,
}

impl RateLimiter {
    pub fn task<S>(&self, builder: RequestBuilder<S>) -> TaskBuilder<S>
    where
        S: BinanceSigner + Unpin,
    {
        TaskBuilder {
            req_builder: builder,
            costs: TaskCosts::new(),
            queue_tx: self.queue_tx.clone(),
        }
    }

    fn recv(&self, mut rx: mpsc::UnboundedReceiver<TaskMessage>) {
        let buckets = self.buckets.clone();
        actix_rt::spawn(async move {
            while let Some(TaskMessage { costs, task_tx }) = rx.next().await {
                let buckets = buckets.clone();
                let res = async move {
                    if let Some(timeout) = Self::timeout(buckets.clone(), &costs).await? {
                        log::debug!("RateLimiter: sleep for {:?}s", timeout);
                        sleep(timeout).await;
                    }
                    Self::set_costs(buckets, &costs).await?;
                    Ok(())
                }
                .await;
                let _ = task_tx.send(res);
            }
        });
    }

    async fn timeout<'a>(
        buckets: Arc<HashMap<BucketName, Mutex<RateLimiterBucket>>>,
        costs: &'a TaskCosts,
    ) -> BinanceResult<Option<Duration>> {
        let mut timeout = Duration::default();

        for (name, cost) in costs {
            let mut bucket = match buckets.get(name) {
                Some(bucket) => bucket.lock().await,
                None => Err(LibError::other(format!(
                    "RateLimiter: undefined bucket - {}",
                    name
                )))?,
            };

            let delay = bucket.delay.duration_since(Instant::now());
            if !delay.is_zero() {
                timeout = delay;
                continue;
            }

            bucket.reset_outdated();
            let new_amount = bucket.amount + cost;

            if new_amount > bucket.limit {
                let elapsed = Instant::now().duration_since(bucket.time_instant);
                let bucket_timeout = bucket.interval - elapsed;

                if bucket_timeout > timeout {
                    timeout = bucket_timeout;
                }
            }
        }

        Ok((!timeout.is_zero()).then(|| timeout))
    }

    async fn set_costs<'a>(
        buckets: Arc<HashMap<BucketName, Mutex<RateLimiterBucket>>>,
        costs: &'a TaskCosts,
    ) -> BinanceResult<()> {
        for (name, cost) in costs {
            let mut bucket = match buckets.get(name) {
                Some(bucket) => bucket.lock().await,
                None => Err(LibError::other(format!(
                    "RateLimiter: undefined bucket - {}",
                    name
                )))?,
            };

            bucket.reset_outdated();
            bucket.amount += cost;
        }

        Ok(())
    }
}

pub(crate) struct RateLimiterBucket {
    time_instant: Instant,
    delay: Instant,
    interval: Duration,
    limit: u32,
    amount: u32,
}

impl Default for RateLimiterBucket {
    fn default() -> Self {
        Self {
            time_instant: Instant::now(),
            delay: Instant::now(),
            interval: Duration::default(),
            limit: 0,
            amount: 0,
        }
    }
}

impl RateLimiterBucket {
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = Instant::now() + delay;
        self
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    fn reset_outdated(&mut self) {
        let elapsed = Instant::now().duration_since(self.time_instant);
        if elapsed > self.interval {
            self.time_instant = Instant::now();
            self.amount = 0;
        }
    }
}

pub(crate) struct TaskBuilder<S>
where
    S: BinanceSigner + Unpin + 'static,
{
    req_builder: RequestBuilder<S>,
    costs: TaskCosts,
    queue_tx: mpsc::UnboundedSender<TaskMessage>,
}

impl<S> TaskBuilder<S>
where
    S: BinanceSigner + Unpin + 'static,
{
    pub fn cost(mut self, key: impl Into<BucketName>, weight: u32) -> Self {
        self.costs
            .entry(key.into())
            .and_modify(|e| *e = weight)
            .or_insert(weight);
        self
    }

    pub fn send<V>(self) -> Task<V>
    where
        V: serde::de::DeserializeOwned + Debug,
    {
        let costs = self.costs.clone();
        let req_builder = self.req_builder;
        let mut queue_tx = self.queue_tx.clone();

        let fut = async move {
            let (task_tx, task_rx) = oneshot::channel::<TaskMessageResult>();

            queue_tx
                .send(TaskMessage { costs, task_tx })
                .await
                .map_err(|_| LibError::other("RateLimiter: task channel was dropped"))?;
            task_rx
                .await
                .map_err(|_| LibError::other("RateLimiter: task channel was dropped"))?
                .map_err(|e| {
                    log::error!("RateLimiter: task err. {:?}", e);
                    e
                })?;

            req_builder.send::<V>().await
        };

        Task {
            fut: fut.boxed_local(),
            costs: self.costs,
        }
    }
}

pub struct Task<V>
where
    V: serde::de::DeserializeOwned + Debug,
{
    fut: Pin<Box<dyn Future<Output = BinanceResult<V>>>>,
    costs: TaskCosts,
}

impl<V> Task<V>
where
    V: serde::de::DeserializeOwned + Debug,
{
    pub fn metadata(&self) -> TaskMetadata {
        TaskMetadata {
            costs: self.costs.clone(),
        }
    }
}

impl<V> Future for Task<V>
where
    V: serde::de::DeserializeOwned + Debug,
{
    type Output = BinanceResult<V>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.fut.poll_unpin(cx)
    }
}

#[derive(Debug)]
pub struct TaskMetadata {
    pub costs: TaskCosts,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::api::spot::ServerTime;
    use crate::{ApiCred, Proxy, SpotApi};

    pub static CCX_BINANCE_API_PREFIX: &str = "CCX_BINANCE_API";

    #[actix_rt::test]
    async fn test_rate_limiter_queue() {
        let proxy = Proxy::from_env_with_prefix(CCX_BINANCE_API_PREFIX);
        let spot_api = SpotApi::new(
            ApiCred::from_env_with_prefix(CCX_BINANCE_API_PREFIX),
            true,
            proxy,
        );

        let rate_limiter = RateLimiterBuilder::default()
            .bucket(
                "interval_1__limit_1",
                RateLimiterBucket::default()
                    .interval(Duration::from_secs(1))
                    .limit(1),
            )
            .bucket(
                "interval_10__limit_2",
                RateLimiterBucket::default()
                    .interval(Duration::from_secs(10))
                    .limit(2),
            )
            .start();

        let instant_now = Instant::now();
        for _i in 1..=8 {
            let task_res = rate_limiter
                .task(spot_api.client.get("/api/v3/time").unwrap())
                .cost("interval_1__limit_1", 1)
                .cost("interval_10__limit_2", 1)
                .send::<ServerTime>()
                .await;
            println!("TASK {:?}", task_res);

            assert!(task_res.is_ok());
        }

        assert!(Instant::now().duration_since(instant_now) >= Duration::from_secs(30));
    }

    #[actix_rt::test]
    async fn test_rate_limiter_metadata() {
        let proxy = Proxy::from_env_with_prefix(CCX_BINANCE_API_PREFIX);
        let spot_api = SpotApi::new(
            ApiCred::from_env_with_prefix(CCX_BINANCE_API_PREFIX),
            true,
            proxy,
        );

        let rate_limiter = RateLimiterBuilder::default()
            .bucket(
                "interval_1__limit_1",
                RateLimiterBucket::default()
                    .interval(Duration::from_secs(1))
                    .limit(1),
            )
            .bucket(
                "interval_10__limit_2",
                RateLimiterBucket::default()
                    .interval(Duration::from_secs(10))
                    .limit(2),
            )
            .start();

        for _i in 1..=8 {
            let task = rate_limiter
                .task(spot_api.client.get("/api/v3/time").unwrap())
                .cost("interval_1__limit_1", 1)
                .cost("interval_10__limit_2", 1)
                .send::<ServerTime>();

            assert_eq!(task.metadata().costs.get("interval_1__limit_1"), Some(&1));
            assert_eq!(task.metadata().costs.get("interval_10__limit_2"), Some(&1));
        }
    }

    #[actix_rt::test]
    async fn test_rate_limiter_delay() {
        let proxy = Proxy::from_env_with_prefix(CCX_BINANCE_API_PREFIX);
        let spot_api = SpotApi::new(
            ApiCred::from_env_with_prefix(CCX_BINANCE_API_PREFIX),
            true,
            proxy,
        );

        let rate_limiter = RateLimiterBuilder::default()
            .bucket(
                "delay_10__interval_1__limit_1",
                RateLimiterBucket::default()
                    .delay(Duration::from_secs(10))
                    .interval(Duration::from_secs(10))
                    .limit(1),
            )
            .start();

        let instant_now = Instant::now();
        for _i in 1..=2 {
            let task_res = rate_limiter
                .task(spot_api.client.get("/api/v3/time").unwrap())
                .cost("delay_10__interval_1__limit_1", 1)
                .send::<ServerTime>()
                .await;

            assert!(task_res.is_ok());
        }

        assert!(Instant::now().duration_since(instant_now) >= Duration::from_secs(20));
    }

    #[actix_rt::test]
    async fn test_rate_limiter_wrong_bucket() {
        let proxy = Proxy::from_env_with_prefix(CCX_BINANCE_API_PREFIX);
        let spot_api = SpotApi::new(
            ApiCred::from_env_with_prefix(CCX_BINANCE_API_PREFIX),
            true,
            proxy,
        );

        let rate_limiter = RateLimiterBuilder::default()
            .bucket(
                "delay_10__interval_1__limit_1",
                RateLimiterBucket::default()
                    .delay(Duration::from_secs(10))
                    .interval(Duration::from_secs(10))
                    .limit(1),
            )
            .start();

        let task_res = rate_limiter
            .task(spot_api.client.get("/api/v3/time").unwrap())
            .cost("interval_1__limit_1", 1)
            .send::<ServerTime>()
            .await;
        assert!(task_res.is_err())
    }
}
