use ckb_types::{bytes::Bytes, packed, prelude::Entity, H160, H256};

use super::schema::{Byte16, Byte20, Byte32, Byte4, Byte8};

impl From<&H160> for Byte20 {
    fn from(val: &H160) -> Self {
        Self::new_unchecked(Bytes::copy_from_slice(val.as_bytes()))
    }
}

impl From<packed::Byte32> for Byte32 {
    fn from(val: packed::Byte32) -> Self {
        Self::new_unchecked(val.as_bytes())
    }
}

impl From<&H256> for Byte32 {
    fn from(val: &H256) -> Self {
        Self::new_unchecked(Bytes::copy_from_slice(val.as_bytes()))
    }
}

macro_rules! number_to_bytes {
    ($NUM: ty, $BYTE: ty) => {
        impl From<$NUM> for $BYTE {
            fn from(val: $NUM) -> Self {
                Self::new_unchecked(Bytes::copy_from_slice(&val.to_le_bytes()))
            }
        }
    };
}

number_to_bytes!(u128, Byte16);
number_to_bytes!(u64, Byte8);
number_to_bytes!(u32, Byte4);
