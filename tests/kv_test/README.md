# Benchmark Instruction

This is an instruction to benchmark RMI for experiments in AirIndex: Versatile Index Tuning Through Data and Storage.

Please follow [dataset](https://github.com/illinoisdata/airindex-public/blob/main/dataset_setup.md) and [query key set](https://github.com/illinoisdata/airindex-public/blob/main/keyset_setup.md) instructions to setup the benchmarking environment. These are examples of environment [reset scripts](https://github.com/illinoisdata/airindex-public/blob/main/reload_examples.md). The following assumes that the datasets are under `/path/to/data/`, key sets are under `/path/to/keyset/` and output files are under `/path/to/output`

## Build

Please follow [instructions here](https://github.com/illinoisdata/RMI/blob/master/README.md) to build the binaries and use RMI optimizer for tuning. To build indexes for all datasets

```
make train_all ROOT=file:///path/to/output STORAGE=storage OUT=out DATA_PATH=file:///path/to/data
```

## Benchmark (5.2)

Benchmark over 40 key set of 1M keys

```
make result_all ROOT=file:///path/to/output RELOAD=~/reload_local.sh STORAGE=storage OUT=out DATA_PATH=file:///path/to/data KEYSET_PATH=file:///path/to/keyset
```

The measurements will be recorded in `/path/to/output/out` folder.

## Build Scalability (5.6)

To measure the build time, run the build script.

```
make scale ROOT=file:///path/to/output DATA_PATH=file:///path/to/data
```

