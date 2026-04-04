mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;

use crate::persist::{
    DatabaseInner,
    loader::{v1::V1Loader, v2::V2Loader, v3::V3Loader, v4::V4Loader, v5::V5Loader, v6::V6Loader},
};

pub trait Loader {
    fn load(&self, bytes: &[u8]) -> Option<DatabaseInner>;
}

#[derive(Debug, Clone)]
pub struct BoxLoader;

impl Loader for BoxLoader {
    fn load(&self, bytes: &[u8]) -> Option<DatabaseInner> {
        V6Loader
            .load(bytes)
            .or_else(|| V5Loader.load(bytes))
            .or_else(|| V4Loader.load(bytes))
            .or_else(|| V3Loader.load(bytes))
            .or_else(|| V2Loader.load(bytes))
            .or_else(|| V1Loader.load(bytes))
    }
}
