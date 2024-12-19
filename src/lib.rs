#![deny(missing_docs)]

//! Faro Sharding is a technique for sharding keys such that adding new destinations does not move data between existing destinations. Only between existing and the new destination.
//!
//! For example, if we have data sharded among 10 servers, when we add an 11th, we don't want to move data from 3 to 5. Faro Sharding guarantees that won't happen.
//!
//! # Example
//! ```
//! use faro_sharding::shard_for;
//! for locations in 4..50 {
//!   assert_eq!(shard_for("foo", locations), 2);
//! }
//! assert_eq!(shard_for("foo", 50), 49);
//! ```
//!
//! # Distribution
//!
//! Faro Sharding shows high quality even distribution among locations. With 1,000,000 keys and 100 locations, the largest has 10229 keys (1.02%) and the smallest has 9761 keys (0.98%).
//!
//! If you want to evaluate the distribution with your own hashing function, you can modify `examples/distribution.rs` to use your hasher.
//!
//! # Algorithm
//!
//! Inspired by [JumpHash](https://arxiv.org/abs/1406.2294), Faro Sharding sequentially hashes the initial key and then the results of those hashes. Different from JumpHash, Faro Sharding only changes the shard for a key when `hash % i == 0`.
//!
//! Faro Sharding can only move a key to a shard on the step corresponding to that shard number. That means that the only movement happens to shard N on step N. Additionally, approximately 1/N of keys are moved on step N, resulting in approximately even distribution.

use std::hash::*;

#[cfg(feature = "seahash")]
mod with_seahash {
    use crate::shard_with_hasher;
    use seahash::*;
    use std::hash::*;

    /// [shard_with_hasher] using [SeaHasher].
    pub fn shard_for(key: impl Hash, total_destinations: u64) -> u64 {
        shard_with_hasher(key, total_destinations, &BuildSeaHasher)
    }

    struct BuildSeaHasher;

    impl BuildHasher for BuildSeaHasher {
        type Hasher = SeaHasher;

        fn build_hasher(&self) -> Self::Hasher {
            SeaHasher::new()
        }
    }
}
#[cfg(feature = "seahash")]
pub use with_seahash::shard_for;

/// Returns the index of the shard for the provided key using the hasher provided.
///
/// If you want a stable output, your hasher must return the exact same hash given the same input. Do not use a hasher with random state.
///
/// # Panics
///
/// If total_destinations == 0.
pub fn shard_with_hasher(
    key: impl Hash,
    total_destinations: u64,
    hasher: &impl BuildHasher,
) -> u64 {
    let mut final_shard = 0;
    let mut last_hash = hasher.hash_one(key);

    assert_ne!(total_destinations, 0, "total_destinations must be > 0");

    for n in 1..total_destinations {
        let hash = hasher.hash_one(last_hash);
        if hash % (n + 1) == 0 {
            final_shard = n;
        }
        last_hash = hash;
    }

    debug_assert!(final_shard < total_destinations);
    final_shard
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::*;

    proptest! {
        #[test]
        fn does_not_move_between_existing_destinations(key: String) {
            let mut last_shard = shard_for(&key, 1);
            for n in 2..=1024 {
                let next_shard = shard_for(&key, n);
                prop_assert!(next_shard == last_shard || next_shard == n - 1);
                last_shard = next_shard;
            }
        }

        #[test]
        fn assigns_to_different_shards(base: String, locations in 16u64..=512u64) {
            let base_shard = shard_for(&base, locations);

            for n in 0..1024 {
                let shard = shard_for(&format!("{base}-{n}"), locations);
                if shard != base_shard {
                    return Ok(());
                }
            }
            prop_assert!(false);
        }
    }

    #[test]
    fn pinning_default_shard() {
        // It is critical that the default `shard_for` implementation always returns the same shards for correctness of our user's systems. Do not remove or change the values here. Adding new entries is ok.
        let shards = maplit::btreemap! {
            "foo" => 49,
            "bar" => 14,
            "baz" => 9,
            "qux" => 60,
            "quux" => 69,
        };

        for (key, expected_shard) in shards {
            assert_eq!(
                shard_for(&key, 73),
                expected_shard,
                "Incorrect shard for {key}"
            );
        }
    }

    #[test]
    fn distributes_to_every_shard() {
        let locations = 16;

        let mut shard_counts = std::collections::HashMap::<_, u64>::new();
        for i in 0..1000 {
            *shard_counts.entry(shard_for(i, locations)).or_default() += 1;
        }

        for l in 0..locations {
            assert!(shard_counts.contains_key(&l), "No entries for shard {l}");
        }
    }
}
