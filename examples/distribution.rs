use faro_sharding::shard_for;
use std::collections::BTreeMap;
use structopt::*;

#[derive(StructOpt)]
struct Options {
    #[structopt(long, default_value = "faro_shard_base")]
    base: String,

    #[structopt(long, default_value = "100")]
    locations: u64,

    #[structopt(long, default_value = "1000000")]
    keys: u64,

    #[structopt(long)]
    print_shard_counts: bool,
}

fn main() {
    let options = Options::from_args();

    println!(
        "Testing distribution of {} keys across {} locations",
        options.keys, options.locations
    );

    let mut location_counts = BTreeMap::<u64, u64>::new();
    for i in 0..options.keys {
        let shard = shard_for(&format!("{}-{i}", options.base), options.locations);
        *location_counts.entry(shard).or_default() += 1u64;
    }

    let min = location_counts
        .iter()
        .min_by_key(|(_, c)| **c)
        .expect("keys > 0");

    let min_percent = 100. * *min.1 as f32 / options.keys as f32;
    println!(
        "Shard {} had the fewest keys - {} ({:.2}%)",
        min.0, min.1, min_percent
    );

    let max = location_counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .expect("keys > 0");

    let max_percent = 100. * *max.1 as f32 / options.keys as f32;
    println!(
        "Shard {} had the most keys - {} ({:.2}%)",
        max.0, max.1, max_percent
    );

    if options.print_shard_counts {
        println!("\nShard counts:");

        for (shard, count) in location_counts {
            println!("  #{shard} - {count}");
        }
    }
}
