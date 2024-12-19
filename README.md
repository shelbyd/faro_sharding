# faro_sharding

Faro Sharding is a technique for sharding keys such that adding new destinations does not move data between existing destinations. Only between existing and the new destination.

For example, if we have data sharded among 10 servers, when we add an 11th, we don't want to move data from 3 to 5. Faro Sharding guarantees that won't happen.

## Example
```rust
use faro_sharding::shard_for;
for locations in 4..51 {
  assert_eq!(shard_for("foo", locations), 3);
}
assert_eq!(shard_for("foo", 51), 50);
```

## Distribution

Faro Sharding shows high quality even distribution among locations. With 1,000,000 keys and 100 locations, the largest has 10332 keys (1.03%) and the smallest has 9854 keys (0.99%).

If you want to evaluate the distribution with your own hashing function, you can modify `examples/distribution.rs` to use your hasher.
