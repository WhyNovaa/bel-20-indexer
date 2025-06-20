use super::*;

pub enum ProcessedData {
    Info {
        block_number: u32,
        block_info: BlockInfo,
    },
    Prevouts {
        to_write: HashMap<OutPoint, TxOut>,
        to_remove: Vec<OutPoint>,
    },
    FullHash {
        addresses: Vec<(FullHash, String)>,
    },
    InscriptionPartials {
        to_remove: Vec<OutPoint>,
        to_write: Vec<(OutPoint, Partials)>,
    },
    InscriptionOffset {
        to_remove: Vec<OutPoint>,
        to_write: Vec<(OutPoint, HashSet<u64>)>,
    },
}

impl ProcessedData {
    pub fn write(self, db: &DB) {
        match self {
            ProcessedData::Info {
                block_number,
                block_info,
            } => {
                db.last_block.set((), block_number);
                db.block_info.set(block_number, block_info);
            }
            ProcessedData::Prevouts {
                to_write,
                to_remove,
            } => {
                db.prevouts.remove_batch(to_remove.into_iter());
                db.prevouts.extend(to_write);
            }
            ProcessedData::FullHash { addresses } => {
                db.fullhash_to_address.extend(addresses);
            }
            ProcessedData::InscriptionPartials {
                to_remove,
                to_write,
            } => {
                db.outpoint_to_partials.remove_batch(to_remove.into_iter());
                db.outpoint_to_partials.extend(to_write);
            }
            ProcessedData::InscriptionOffset {
                to_remove,
                to_write,
            } => {
                db.outpoint_to_inscription_offsets
                    .remove_batch(to_remove.into_iter());
                db.outpoint_to_inscription_offsets.extend(to_write);
            }
        }
    }
}
