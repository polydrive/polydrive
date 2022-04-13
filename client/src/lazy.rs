use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Endpoint;

#[tonic::async_trait]
pub trait TonicClient
    where
        Self: Sized,
{
    async fn try_connect(dest: Endpoint) -> Result<Self, tonic::transport::Error>;
}

#[derive(Debug, Clone)]
enum LazyClientImpl<Client> {
    NotConnected(Box<Endpoint>),
    Connected(Client),
}

// We use an Arc<RwLock<_>> for two reasons
//   1. Arc so that clones are trivial and synchronous
//   2. RwLock so that interior mutations can "propagate" across all copies of this Client
//
// This ensures that we don't do the handshaking work more than 1 time ever.
// While is was technically possible to make a LazyClient before without the RwLock, after
// a clone on that client, connecting from LazyClient variable `a` would be separate from connection from LazyClient variable `b`.
//
// In other words, it's inherently necessary to mutate &self, but we want concurrency, so we use an RwLock for optimistic reads.
#[derive(Debug, Clone)]
pub struct LazyClient<Client>(Arc<RwLock<LazyClientImpl<Client>>>);

impl<Client> LazyClientImpl<Client>
    where
        Client: TonicClient + Clone,
{
    fn create<Dest: Into<Endpoint>>(dest: Dest) -> Self {
        Self::NotConnected(Box::new(dest.into()))
    }
}

impl<Client> LazyClient<Client>
    where
        Client: TonicClient + Clone,
{
    pub fn create<Dest: Into<Endpoint>>(dest: Dest) -> Self {
        Self(Arc::new(RwLock::new(LazyClientImpl::create(dest))))
    }

    pub async fn conn(&self) -> Result<Client, tonic::transport::Error> {
        if let LazyClientImpl::Connected(client) = &*self.0.read().await {
            // fast path, don't need a write lock
            return Ok(client.clone());
        }

        log::debug!("LazyClient intializing first connection...");

        let mut inner_write = self.0.write().await;

        match &*inner_write {
            LazyClientImpl::Connected(_) => {
                unreachable!("we determined above that self is NotConnected")
            }
            LazyClientImpl::NotConnected(dest) => {
                let client = Client::try_connect(*dest.clone()).await?;

                *inner_write = LazyClientImpl::Connected(client.clone());

                Ok(client)
            }
        }
    }
}