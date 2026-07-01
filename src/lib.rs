// SPDX-License-Identifier: MIT
#![no_std]
#![allow(dead_code, deprecated, unused_imports, clippy::too_many_arguments)]

mod contract;
mod errors;
mod events;
mod storage;
mod types;

pub use crate::errors::*;

pub use contract::MergeMintContractClient;

#[cfg(test)]
mod test;
