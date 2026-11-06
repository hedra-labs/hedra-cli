//! The session: a JWT from the caller's minter, reused for every call an
//! invocation opens until a minute before it expires, and the one channel to
//! the broker those calls share.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use tokio::sync::Mutex;
use tokio::time::Instant;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;

use crate::pb::hedra::sandbox::v1::session_service_client::SessionServiceClient;
use crate::stream::transport::{self, Breaker, TransportConfig, TransportError};

/// A session JWT and where to use it.
#[derive(Clone, Debug)]
pub struct Minted {
    pub token: String,
    pub expires_at: SystemTime,
    /// The mint answer's `Date`, so the token's life is measured from when it
    /// was issued rather than from this machine's clock.
    pub date: Option<SystemTime>,
    /// `host:port`, with no scheme.
    pub broker_url: String,
}

pub type MintFuture<'a, E> = Pin<Box<dyn Future<Output = Result<Minted, E>> + Send + 'a>>;

/// Mints a session JWT for one sandbox replica.
pub trait SessionMinter: Send + Sync {
    type Error: Send;
    fn mint(&self) -> MintFuture<'_, Self::Error>;
}

/// How a session could not be made ready.
#[derive(Debug)]
pub enum SessionError<E> {
    Mint(E),
    Transport(TransportError),
}

/// A token is refreshed this long before it expires.
const REFRESH_MARGIN: Duration = Duration::from_secs(60);
/// The broker accepts a session JWT for at most this long.
const MAX_LIFE: Duration = Duration::from_secs(10 * 60);

/// How long a minted token lives from now.
pub(crate) fn life(minted: &Minted, now: SystemTime) -> Duration {
    match minted.date {
        Some(date) => minted.expires_at.duration_since(date).unwrap_or_default(),
        None => minted
            .expires_at
            .duration_since(now)
            .unwrap_or_default()
            .min(MAX_LIFE),
    }
}

struct State {
    valid_until: Option<Instant>,
    stale: bool,
    channel: Option<(String, Channel, Breaker)>,
}

/// The session one invocation's calls share.
pub struct Session<M: SessionMinter> {
    minter: M,
    transport: TransportConfig,
    token: Arc<RwLock<String>>,
    state: Mutex<State>,
    /// The proxy the channel goes through, for the caller's lines.
    proxy: RwLock<Option<String>>,
}

/// Attaches the session's current token to each call.
#[derive(Clone)]
pub struct Bearer(Arc<RwLock<String>>);

impl tonic::service::Interceptor for Bearer {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let token = self.0.read().map(|t| t.clone()).unwrap_or_default();
        let value = format!("Bearer {token}").parse().map_err(|_| {
            tonic::Status::unauthenticated("the session token is not valid metadata")
        })?;
        req.metadata_mut().insert("authorization", value);
        Ok(req)
    }
}

pub type Client = SessionServiceClient<InterceptedService<Channel, Bearer>>;

impl<M: SessionMinter> Session<M> {
    pub fn new(minter: M, transport: TransportConfig) -> Self {
        Self {
            minter,
            transport,
            token: Arc::new(RwLock::new(String::new())),
            state: Mutex::new(State {
                valid_until: None,
                stale: true,
                channel: None,
            }),
            proxy: RwLock::new(None),
        }
    }

    /// Makes the next call's token one that is not near its end, minting
    /// when it is, and returns a client on the session's channel. Mints run
    /// one at a time; open calls keep the token they opened with.
    pub async fn client(&self) -> Result<Client, SessionError<M::Error>> {
        self.client_with_breaker().await.map(|(client, _)| client)
    }

    /// `client`, with the breaker of the channel it is on.
    pub(crate) async fn client_with_breaker(
        &self,
    ) -> Result<(Client, Breaker), SessionError<M::Error>> {
        let mut state = self.state.lock().await;
        let fresh = !state.stale
            && state
                .valid_until
                .is_some_and(|until| Instant::now() + REFRESH_MARGIN < until);
        if !fresh {
            let minted = self.minter.mint().await.map_err(SessionError::Mint)?;
            let life = life(&minted, SystemTime::now());
            // The channel first, so a failure leaves the session stale.
            let (channel, breaker) = match &state.channel {
                Some((url, channel, breaker)) if *url == minted.broker_url => {
                    (channel.clone(), breaker.clone())
                }
                _ => transport::channel_with_breaker(&minted.broker_url, &self.transport)
                    .map_err(SessionError::Transport)?,
            };
            if let Ok(mut token) = self.token.write() {
                *token = minted.token.clone();
            }
            if let Ok(mut proxy) = self.proxy.write() {
                *proxy =
                    transport::proxy_for(&self.transport, &minted.broker_url).map(str::to_string);
            }
            state.channel = Some((minted.broker_url, channel, breaker));
            state.valid_until = Some(Instant::now() + life);
            state.stale = false;
        }
        let Some((_, channel, breaker)) = &state.channel else {
            unreachable!("a fresh session has a channel")
        };
        let client =
            SessionServiceClient::with_interceptor(channel.clone(), Bearer(self.token.clone()));
        Ok((client, breaker.clone()))
    }

    /// The broker the session's channel goes to, once minted.
    pub async fn broker_url(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.channel.as_ref().map(|(url, _, _)| url.clone())
    }

    /// The proxy the session's channel goes through: none before the first
    /// mint, or when `no_proxy` names the broker.
    pub fn proxy(&self) -> Option<String> {
        self.proxy.read().ok().and_then(|p| p.clone())
    }

    /// Makes the next call mint first, as after a refusal for the token.
    pub async fn invalidate(&self) {
        self.state.lock().await.stale = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn minted(life: u64, date_skew: Option<i64>) -> Minted {
        let now = SystemTime::now();
        let date = date_skew.map(|s| {
            if s >= 0 {
                now + Duration::from_secs(s as u64)
            } else {
                now - Duration::from_secs((-s) as u64)
            }
        });
        Minted {
            token: "t".into(),
            expires_at: date.unwrap_or(now) + Duration::from_secs(life),
            date,
            broker_url: "127.0.0.1:1".into(),
        }
    }

    #[test]
    fn life_is_measured_from_the_answers_date() {
        // A clock an hour behind the server still sees ten minutes.
        let m = minted(600, Some(3_600));
        assert_eq!(life(&m, SystemTime::now()), Duration::from_secs(600));
        // A clock an hour ahead, which would see the token already expired.
        let m = minted(600, Some(-3_600));
        assert_eq!(life(&m, SystemTime::now()), Duration::from_secs(600));
    }

    #[test]
    fn without_a_date_life_is_capped_at_the_brokers_maximum() {
        let mut m = minted(0, None);
        m.expires_at = SystemTime::now() + Duration::from_secs(3_600);
        assert!(life(&m, SystemTime::now()) <= MAX_LIFE);
        m.expires_at = SystemTime::now() - Duration::from_secs(5);
        assert_eq!(life(&m, SystemTime::now()), Duration::ZERO);
    }

    struct Counting(AtomicUsize, u64);

    impl SessionMinter for Counting {
        type Error = ();
        fn mint(&self) -> MintFuture<'_, ()> {
            self.0.fetch_add(1, Ordering::SeqCst);
            let m = minted(self.1, Some(0));
            Box::pin(async move { Ok(m) })
        }
    }

    #[tokio::test]
    async fn one_mint_serves_calls_until_a_minute_before_expiry() {
        let s = Session::new(
            Counting(AtomicUsize::new(0), 600),
            TransportConfig::default(),
        );
        for _ in 0..3 {
            s.client().await.ok().unwrap();
        }
        assert_eq!(s.minter.0.load(Ordering::SeqCst), 1);
        s.invalidate().await;
        s.client().await.ok().unwrap();
        assert_eq!(s.minter.0.load(Ordering::SeqCst), 2);
    }

    /// Answers each mint with the next broker address, repeating the last.
    struct Brokers(AtomicUsize, Vec<&'static str>);

    impl SessionMinter for Brokers {
        type Error = ();
        fn mint(&self) -> MintFuture<'_, ()> {
            let i = self.0.fetch_add(1, Ordering::SeqCst);
            let mut m = minted(600, Some(0));
            m.broker_url = self.1[i.min(self.1.len() - 1)].into();
            Box::pin(async move { Ok(m) })
        }
    }

    #[tokio::test]
    async fn a_broker_address_that_cannot_be_used_leaves_the_session_stale() {
        // A first mint whose channel fails: the next call mints again.
        let s = Session::new(
            Brokers(AtomicUsize::new(0), vec!["no port", "127.0.0.1:1"]),
            TransportConfig::default(),
        );
        assert!(matches!(s.client().await, Err(SessionError::Transport(_))));
        s.client().await.ok().unwrap();
        assert_eq!(s.minter.0.load(Ordering::SeqCst), 2);

        // A re-mint naming a new broker whose channel fails never falls back
        // to the old broker's channel.
        let s = Session::new(
            Brokers(
                AtomicUsize::new(0),
                vec!["127.0.0.1:1", "no port", "127.0.0.1:2"],
            ),
            TransportConfig::default(),
        );
        s.client().await.ok().unwrap();
        s.invalidate().await;
        assert!(s.client().await.is_err());
        s.client().await.ok().unwrap();
        assert_eq!(s.minter.0.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn a_token_within_a_minute_of_expiry_is_minted_again() {
        let s = Session::new(
            Counting(AtomicUsize::new(0), 50),
            TransportConfig::default(),
        );
        s.client().await.ok().unwrap();
        s.client().await.ok().unwrap();
        assert_eq!(s.minter.0.load(Ordering::SeqCst), 2);
    }
}
