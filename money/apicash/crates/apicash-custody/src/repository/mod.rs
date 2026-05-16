//! Custody persistence.

mod custody_repository;

pub use custody_repository::{
    CustodyRepository, InMemoryCustodyRepository, PostgresCustodyRepository,
};
