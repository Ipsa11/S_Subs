# The `benchmark block` command

The whole benchmarking process in Substrate aims to predict the resource usage of an unexecuted block. This command
measures how accurate this prediction was by executing a block and comparing the predicted weight to its actual resource
usage. It can be used to measure the accuracy of the pallet benchmarking.

## Substrate

It is also possible to try the procedure in Substrate, although it's a bit boring.

First you need to create some blocks with either a local or dev chain. This example will use the standard development
spec. Pick a non existing directory where the chain data will be stored, eg `/tmp/dev`.
```sh
cargo run --profile=production -- --dev -d /tmp/dev
```
You should see after some seconds that it started to produce blocks:
```pre
…
✨ Imported #1 (0x801d…9189)
…
```
You can now kill the node with `Ctrl+C`. Then measure how long it takes to execute these blocks:
```sh
cargo run --profile=production -- benchmark block --from 1 --to 1 --dev -d /tmp/dev --pruning archive
```
This will benchmark the first block. If you killed the node at a later point, you can measure multiple blocks.
```pre
Block 1 with     1 tx used  72.04% of its weight (     4,945,664 of      6,864,702 ns)
```

In this example the block used ~72% of its weight. The benchmarking therefore over-estimated the effort to execute the
block. Since this block is empty, its not very interesting.

## Arguments

- `--from` Number of the first block to measure (inclusive).
- `--to` Number of the last block to measure (inclusive).
- `--repeat` How often each block should be measured.
- [`--db`]
- [`--pruning`]

License: Apache-2.0

<!-- LINKS -->

[`--db`]: ../shared/README.md#arguments
[`--pruning`]: ../shared/README.md#arguments
