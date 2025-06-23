use super::*;

mod structs;
pub use structs::*;

rocksdb_wrapper::generate_db_code! {
    block_info: u32 => BlockInfo,
    prevouts: UsingConsensus<OutPoint> => UsingConsensus<TxOut>,
    outpoint_to_partials: UsingConsensus<OutPoint> => Partials,
    outpoint_to_inscription_offsets: UsingConsensus<OutPoint> => UsingSerde<HashSet<u64>>,
    last_block: () => u32,
    fullhash_to_address: FullHash => String,
    // NEW
    last_inscription_number: () => u64,
}
/*
generate_db_code! {
    +partials: UsingConsensus<OutPoint> => UsingSerde<Partial>,
    address_to_stats: FullHash => UsingSerde<AddressStat>,
    address_location_to_inscription: AddressLocation => UsingSerde<InscriptionMeta>,
    last_inscription_number: () => u64,
    utxos_cache: AddressOutPoint => (),
    last_block: () => u32,
    genesis_to_location: InscriptionId => AddressLocation,
    scripthash_to_address: FullHash => String,
    +outpoint_to_inscription_offsets: UsingConsensus<OutPoint> => UsingSerde<HashSet<(Offset, Number)>>,
}
*/

impl DB {}
