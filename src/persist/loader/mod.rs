mod v1;
mod v2;

use crate::persist::{
    DatabaseInner,
    loader::{v1::V1Loader, v2::V2Loader},
};

pub trait Loader {
    fn load(&self, bytes: &[u8]) -> Option<DatabaseInner>;
}

#[derive(Debug, Clone)]
pub struct BoxLoader;

impl Loader for BoxLoader {
    fn load(&self, bytes: &[u8]) -> Option<DatabaseInner> {
        V2Loader.load(bytes).or_else(|| V1Loader.load(bytes))
    }
}
