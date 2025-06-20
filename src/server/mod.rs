use super::*;

mod structs;
pub mod threads;
pub use structs::*;

pub struct Server {
    pub db: Arc<DB>,
    pub event_sender: tokio::sync::broadcast::Sender<ServerEvent>,
    pub raw_event_sender: kanal::Sender<RawServerEvent>,
    pub token: WaitToken,
    pub holders: Arc<Holders>,
    pub indexer: Arc<nint_blk::Indexer>,
}

impl Server {
    pub fn new(
        db_path: &str,
    ) -> anyhow::Result<(
        kanal::Receiver<RawServerEvent>,
        tokio::sync::broadcast::Sender<ServerEvent>,
        Self,
    )> {
        let (raw_tx, raw_rx) = kanal::unbounded();
        let (tx, _) = tokio::sync::broadcast::channel(30_000);
        let token = WaitToken::default();
        let db = Arc::new(DB::open(db_path));

        let coin = match (BLOCKCHAIN.as_str(), *NETWORK) {
            ("bells", Network::Bellscoin) => "bellscoin",
            ("bells", Network::Testnet) => "bellscoin-testnet",
            ("doge", Network::Bellscoin) => "dogecoin",
            ("doge", Network::Testnet) => "dogecoin-testnet",
            _ => "bellscoin",
        }
        .to_string();

        let indexer = nint_blk::Indexer {
            coin,
            last_height: db.last_block.get(()).unwrap_or_default(),
            path: BLK_DIR.to_string(),
            reorg_max_len: REORG_CACHE_MAX_LEN,
            rpc_auth: nint_blk::Auth::UserPass(USER.to_string(), PASS.to_string()),
            rpc_url: URL.to_string(),
            token: token.clone(),
            index_dir_path: INDEX_DIR.to_string(),
        };

        let server = Self {
            holders: Arc::new(Holders::init(&db)),
            raw_event_sender: raw_tx.clone(),
            token,
            event_sender: tx.clone(),
            indexer: Arc::new(indexer),
            db,
        };

        Ok((raw_rx, tx, server))
    }

    pub fn load_addresses(
        &self,
        keys: impl IntoIterator<Item = FullHash>,
    ) -> anyhow::Result<AddressesFullHash> {
        let keys = keys.into_iter().collect::<HashSet<_>>();

        Ok(AddressesFullHash::new(
            self.db
                .fullhash_to_address
                .multi_get_kv(keys.iter(), false)
                .into_iter()
                .map(|(k, v)| (*k, v))
                .collect(),
        ))
    }
}
