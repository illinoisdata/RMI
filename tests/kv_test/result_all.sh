set -e
  
ROOT=$1
RELOAD=$2
STORAGE=$3
OUT=$4
OUT_PATH=${ROOT}/${OUT}
STORAGE_PATH=${ROOT}/${STORAGE}

DATASET_NAMES=(
  "wiki_ts_200M_uint64"
  "osm_cellids_800M_uint64"
  "fb_200M_uint64"
  "books_800M_uint64"
  "gmm_k100_800M_uint64"
)

mkdir -p ${OUT_PATH}

echo "Testing mmap"
for ((i = 0; i < ${#DATASET_NAMES[@]}; i++)) do
  for ((j = 0; j < 40; j++)) do
    dataset_name="${DATASET_NAMES[$i]}"
    echo ">>> ${dataset_name} ${j}"
    bash ${RELOAD}
    ${ROOT}/binary/main_${dataset_name}_rmi_mmap --data_path=${HOME}/data/${dataset_name} --key_path=${HOME}/keyset/${dataset_name}_ks_${j} --rmi_data_path=${STORAGE_PATH}/${dataset_name}_rmi_mmap/rmi_data --out_path=${OUT_PATH}/out_main_${dataset_name}_rmi_mmap.txt 2>& 1 | tee log.txt
  done
done
