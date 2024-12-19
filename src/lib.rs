#![deny(missing_docs)]

//! Faro Sharding is a technique for sharding keys such that adding new destinations does not move data between existing destinations. Only between existing and the new destination.
//!
//! For example, if we have data sharded among 10 servers, when we add an 11th, we don't want to move data from 3 to 5. Faro Sharding guarantees that won't happen.
//!
//! # Example
//! ```
//! use faro_sharding::shard_for;
//! for locations in 4..51 {
//!   assert_eq!(shard_for("foo", locations), 3);
//! }
//! assert_eq!(shard_for("foo", 51), 50);
//! ```
//!
//! # Distribution
//!
//! Faro Sharding shows high quality even distribution among locations. With 1,000,000 keys and 100 locations, the largest has 10332 keys (1.03%) and the smallest has 9854 keys (0.99%).
//!
//! If you want to evaluate the distribution with your own hashing function, you can modify `examples/distribution.rs` to use your hasher.

use std::hash::*;

#[cfg(feature = "seahash")]
mod with_seahash {
    use seahash::*;
    use std::hash::*;

    /// [shard_with_hasher] using [SeaHasher].
    pub fn shard_for(key: impl Hash, total_destinations: u64) -> u64 {
        crate::shard_with_hasher(key, total_destinations, &BuildSeaHasher)
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

    for n in 2..total_destinations {
        let hash = hasher.hash_one(last_hash);
        if hash % n == 0 {
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
            "foo" => 50,
            "bar" => 15,
            "baz" => 10,
            "qux" => 61,
            "quux" => 70,
        };

        for (key, expected_shard) in shards {
            assert_eq!(
                shard_for(&key, 73),
                expected_shard,
                "Incorrect shard for {key}"
            );
        }
    }
}
