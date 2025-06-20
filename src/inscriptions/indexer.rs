use bitcoin_hashes::sha256d;

use super::*;

pub struct InscriptionIndexer {
    server: Arc<Server>,
    pub reorg_cache: Option<Arc<parking_lot::Mutex<ReorgCache>>>,
}

#[derive(Default)]
pub struct DataToWrite {
    pub processed: Vec<ProcessedData>,
}

impl InscriptionIndexer {
    pub fn new(
        server: Arc<Server>,
        reorg_cache: Option<Arc<parking_lot::Mutex<ReorgCache>>>,
    ) -> Self {
        Self {
            reorg_cache,
            server,
        }
    }

    pub fn handle(
        &self,
        block_height: u32,
        block: nint_blk::proto::block::Block,
    ) -> anyhow::Result<()> {
        let mut to_write = DataToWrite::default();

        self.handle_block(&mut to_write, block_height, block)?;

        // write/remove data from block
        for data in to_write.processed {
            data.write(&self.server.db);
        }

        Ok(())
    }

    fn handle_block(
        &self,
        to_write: &mut DataToWrite,
        block_height: u32,
        block: nint_blk::proto::block::Block,
    ) -> anyhow::Result<()> {
        let current_hash = block.header.hash;

        let mut last_history_id = self.server.db.last_history_id.get(()).unwrap_or_default();

        if let Some(cache) = self.reorg_cache.as_ref() {
            debug!("Syncing block: {} ({})", current_hash, block_height);
            cache.lock().new_block(block_height, last_history_id);
        }

        let block_info = BlockInfo {
            created: block.header.value.timestamp,
            hash: current_hash.into(),
        };

        let prev_block_height = block_height.checked_sub(1).unwrap_or_default();
        let prev_block_proof = self
            .server
            .db
            .proof_of_history
            .get(prev_block_height)
            .unwrap_or(*DEFAULT_HASH);

        let outpoint_fullhash_to_address = block
            .txs
            .iter()
            .flat_map(|x| &x.value.outputs)
            .filter_map(|x| {
                x.script.address.as_ref().map(|address| {
                    let fullhash: FullHash = sha256d::Hash::hash(&x.out.script_pubkey).into();
                    (fullhash, address.to_owned())
                })
            })
            .collect::<HashMap<_, _>>();

        to_write.processed.push(ProcessedData::Info {
            block_number: block_height,
            block_info,
        });

        let prevouts =
            utils::process_prevouts(self.server.db.clone(), &block, &mut to_write.processed)?;

        to_write.processed.push(ProcessedData::FullHash {
            addresses: outpoint_fullhash_to_address
                .iter()
                .map(|(fullhash, address)| (*fullhash, address.to_owned()))
                .collect(),
        });

        if block_height < *START_HEIGHT {
            return Ok(());
        }

        if let Some(cache) = self.reorg_cache.as_ref() {
            prevouts.iter().for_each(|(key, value)| {
                cache.lock().removed_prevout(*key, value.clone());
            });
        }


        let last_inscription_number = self.server.db.last_inscription_number.get(()).unwrap_or_default();

        let mut parser = Parser {
            server: &self.server,
            reorg_cache: self.reorg_cache.clone(),
            last_inscription_number,
        };

        parser.parse_block(block_height, block, &prevouts, &mut to_write.processed);

        let mut fullhash_to_load = HashSet::new();

        let rest_addresses: AddressesFullHash = self
            .server
            .db
            .fullhash_to_address
            .multi_get_kv(
                fullhash_to_load
                    .iter()
                    .filter(|x| !outpoint_fullhash_to_address.contains_key(x)),
                false,
            )
            .into_iter()
            .map(|(k, v)| (*k, v))
            .chain(outpoint_fullhash_to_address)
            .collect::<HashMap<_, _>>()
            .into();

        parser.write_inscription_number();

        Ok(())
    }
}

#[derive(Debug)]
pub enum ParsedInscriptionResult {
    None,
    Partials,
    Single(InscriptionTemplate),
    Many(Vec<InscriptionTemplate>),
}
