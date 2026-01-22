use super::*;

use p2panda_core::cbor::{decode_cbor, encode_cbor};
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

/// Wrapper type to handle keys and values using bincode serialization
#[allow(unused)]
#[derive(Debug)]
pub struct Binary<T>(pub T);

impl<T> Value for Binary<T>
where
    T: Debug + Serialize + DeserializeOwned,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_cbor(data).expect("from_bytes couldn't decode")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        encode_cbor(value).expect("as_bytes couldn't encode")
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bincode<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Key for Binary<T>
where
    T: Debug + Serialize + DeserializeOwned + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}
