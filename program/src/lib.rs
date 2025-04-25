pub mod assertions;
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod info;
// pub mod utils;
pub mod mtree;

pub use solana_program;

solana_program::declare_id!("5btvfbzMrGv3WB4h47NXpophMLKqGEEDwGDQgkr8PMF2");
