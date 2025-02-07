# Benchmark Instruction

This is an instruction to benchmark RMI for experiments in AirIndex: Versatile Index Tuning Through Data and Storage.

Please follow [dataset](https://github.com/illinoisdata/airindex-public/blob/main/dataset_setup.md) and [query key set](https://github.com/illinoisdata/airindex-public/blob/main/keyset_setup.md) instructions to setup the benchmarking environment. These are examples of environment [reset scripts](https://github.com/illinoisdata/airindex-public/blob/main/reload_examples.md). The following assumes that the datasets are under `/path/to/data/`, key sets are under `/path/to/keyset/` and output files are under `/path/to/output`

## Build

Please follow [instructions here](https://github.com/illinoisdata/RMI/blob/master/README.md) to build the binaries and use RMI optimizer for tuning. To build indexes for all datasets

Edit `train_all.sh` to change RMI configuration for each dataset.

```
make train_all ROOT=/path/to/output STORAGE=storage OUT=out DATA_PATH=/path/to/data
```

## Benchmark (6.2 & 6.4)

Benchmark over 40 key set of 1M keys

```
make result_all ROOT=/path/to/output RELOAD=~/reload_local.sh STORAGE=storage OUT=out DATA_PATH=/path/to/data KEYSET_PATH=/path/to/keyset
```

The measurements will be recorded in `/path/to/output/out` folder.

## All Configurations (6.5)

The RMI optimizer should output 10 different configurations varying model type and size. Select each of them and edit `train_all.sh` to build and benchmark one by one.

## Build Scalability (6.7)

To measure the build time, run the build script.

```
make scale ROOT=/path/to/output DATA_PATH=/path/to/data
```

