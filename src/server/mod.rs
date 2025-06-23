use super::*;

mod structs;
pub mod threads;
pub use structs::*;

pub struct Server {
    pub db: Arc<DB>,
    pub token: WaitToken,
    pub indexer: Arc<nint_blk::Indexer>,
}

impl Server {
    pub fn new(
        db_path: &str,
    ) -> anyhow::Result<Self> {
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
            reorg_max_len: 0, // todo set reorg
            rpc_auth: nint_blk::Auth::UserPass(USER.to_string(), PASS.to_string()),
            rpc_url: URL.to_string(),
            token: token.clone(),
            index_dir_path: INDEX_DIR.to_string(),
        };

        let server = Self {
            token,
            indexer: Arc::new(indexer),
            db,
        };

        Ok(server)
    }
}
