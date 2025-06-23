use bellscoin::consensus;

use super::*;
use inscriptions::structs::Part;

#[derive(Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LowerCaseTokenTick(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for LowerCaseTokenTick {
    fn from(value: T) -> Self {
        LowerCaseTokenTick(
            String::from_utf8_lossy(value.as_ref())
                .to_lowercase()
                .as_bytes()
                .to_vec(),
        )
    }
}

impl std::ops::Deref for LowerCaseTokenTick {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for LowerCaseTokenTick {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl rocksdb_wrapper::Pebble for LowerCaseTokenTick {
    type Inner = Self;

    fn get_bytes(v: &Self::Inner) -> Cow<[u8]> {
        Cow::Borrowed(&v.0)
    }

    fn from_bytes(v: Cow<[u8]>) -> anyhow::Result<Self::Inner> {
        Ok(Self(v.into_owned()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct AddressLocation {
    pub address: FullHash,
    pub location: Location,
}

impl AddressLocation {
    pub fn search_with_offset(address: FullHash, outpoint: OutPoint) -> RangeInclusive<Self> {
        let start = Self {
            address,
            location: Location {
                outpoint,
                offset: 0,
                number: 0,
            },
        };
        let end = Self {
            address,
            location: Location {
                outpoint,
                offset: u64::MAX,
                number: u64::MAX,
            },
        };

        start..=end
    }

    pub fn search(address: FullHash, offset: Option<OutPoint>) -> RangeInclusive<Self> {
        if let Some(offset) = offset {
            return Self::search_offset(address, offset);
        }

        let start = Self {
            address,
            location: Location {
                outpoint: OutPoint {
                    txid: Txid::all_zeros(),
                    vout: 0,
                },
                offset: 0,
                number: 0,
            },
        };
        let end = Self {
            address,
            location: Location {
                outpoint: OutPoint {
                    txid: Txid::from_byte_array([u8::MAX; 32]),
                    vout: u32::MAX,
                },
                offset: u64::MAX,
                number: u64::MAX,
            },
        };

        start..=end
    }

    fn search_offset(address: FullHash, offset: OutPoint) -> RangeInclusive<Self> {
        let start = Self {
            address,
            location: Location {
                outpoint: offset,
                offset: 0,
                number: 0,
            },
        };
        let end = Self {
            address,
            location: Location {
                outpoint: OutPoint {
                    txid: Txid::from_byte_array([u8::MAX; 32]),
                    vout: u32::MAX,
                },
                offset: u64::MAX,
                number: u64::MAX,
            },
        };

        start..=end
    }
}

impl rocksdb_wrapper::Pebble for AddressLocation {
    type Inner = Self;

    fn get_bytes(v: &Self::Inner) -> Cow<[u8]> {
        let mut result = Vec::with_capacity(32 + 44);

        result.extend(v.address);

        result.extend(consensus::serialize(&v.location.outpoint));
        result.extend(v.location.offset.to_be_bytes());
        result.extend(v.location.number.to_be_bytes());

        Cow::Owned(result)
    }

    fn from_bytes(v: Cow<[u8]>) -> anyhow::Result<Self::Inner> {
        let address = v[..32].try_into().anyhow()?;
        let outpoint: OutPoint = consensus::deserialize(&v[32..32 + 36])?;
        let offset = u64::from_be_bytes(v[32 + 36..32 + 36 + 8].try_into().anyhow()?);
        let number = u64::from_be_bytes(v[32 + 36 + 8..].try_into().anyhow()?);

        Ok(Self {
            address,
            location: Location { outpoint, offset, number },
        })
    }
}

#[derive(Clone, Debug)]
pub struct Partials {
    pub inscription_index: u32,
    pub genesis_txid: Txid,
    pub parts: Vec<Part>,
}

impl rocksdb_wrapper::Pebble for Partials {
    type Inner = Self;

    fn get_bytes(v: &Self::Inner) -> Cow<[u8]> {
        let mut buffer = vec![];
        buffer.extend(v.inscription_index.to_be_bytes().to_vec());
        buffer.extend_from_slice(&bellscoin::consensus::serialize(&v.genesis_txid));

        for part in &v.parts {
            buffer.extend([part.is_tapscript as u8]);
            let script_len = part.script_buffer.len() as u32;
            buffer.extend(script_len.to_be_bytes().to_vec());
            buffer.extend(part.script_buffer.clone());
        }

        Cow::Owned(buffer)
    }

    fn from_bytes(v: Cow<[u8]>) -> anyhow::Result<Self::Inner> {
        let inscription_index = u32::from_be_bytes(v[..4].try_into()?);
        let genesis_txid: Txid = bellscoin::consensus::deserialize(&v[4..36])?;
        let mut parts = vec![];
        let mut offset = 4 + 32;
        while offset != v.len() {
            let is_tapscript = v[offset] == 1;
            offset += 1;
            let script_len = u32::from_be_bytes(v[offset..offset + 4].try_into()?) as usize;
            offset += 4;
            let script_buffer = v[offset..offset + script_len].to_vec();

            parts.push(Part {
                is_tapscript,
                script_buffer,
            });
        }

        Ok(Self {
            genesis_txid,
            inscription_index,
            parts,
        })
    }
}

#[derive(Clone, Copy)]
pub struct BlockInfo {
    pub hash: BlockHash,
    pub created: u32,
}

impl rocksdb_wrapper::Pebble for BlockInfo {
    type Inner = Self;

    fn get_bytes(v: &Self::Inner) -> Cow<[u8]> {
        Cow::Owned(
            [
                v.hash.to_byte_array().as_slice(),
                v.created.to_be_bytes().as_slice(),
            ]
            .concat(),
        )
    }

    fn from_bytes(v: Cow<[u8]>) -> anyhow::Result<Self::Inner> {
        let hash = BlockHash::from_byte_array(v[0..32].try_into()?);
        let created = u32::from_be_bytes(v[32..].try_into()?);

        Ok(Self { created, hash })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Partial {
    pub total_fee: u64,
    pub inscription_idx: u32,
    pub outpoints: Vec<OutPoint>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct AddressStat {
    pub amount: u64,
    /*/// if negative user unlocked more, than he locked, if positive user locked more than he unlocked (total unlocked balance = total - amount - diff_amount )
    pub diff_amount: i64,*/
    pub count: u64,
    pub total: u64,
}
