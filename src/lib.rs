// SPDX-License-Identifier: MIT
#![no_std]

pub mod errors;
mod contract;
mod errors;
mod events;
mod storage;
mod types;

pub use contract::MergeMintContractClient;

#[cfg(test)]
mod test;
