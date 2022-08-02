set -e
echo "Training with compiler flags ${CXX} ${CXXFLAGS} ${CXXBACKFLAGS}"

ROOT=$1
STORAGE=$2
DATA_PATH="${HOME}/data"
STORAGE_PATH="${ROOT}/${STORAGE}"
BINARY_PATH="${ROOT}/binary"

echo ${STORAGE_PATH}
mkdir -p ${ROOT}
mkdir -p ${STORAGE_PATH}
mkdir -p ${BINARY_PATH}

DATASET_NAMES=(
 "wiki_ts_200M_uint64"
 "osm_cellids_800M_uint64"
 "fb_200M_uint64"
 "books_800M_uint64"
 "gmm_k100_800M_uint64"
)

#RMI_CONFIGS=( # configurations obtained from rmi optimizer
# "linear_spline,linear  16777216"
# "cubic,linear  16777216"
# "robust_linear,linear  16777216"
# "linear_spline,linear  16777216"
# "linear_spline,linear 1048576"
#)

RMI_CONFIGS=( # configurations obtained from rmi optimizer
 "linear_spline,linear  128"
 "robust_linear,linear  128"
 "robust_linear,linear  128"
 "linear_spline,linear  128"
 "radix,linear 128"
)

echo "Training mmap"
for ((i = 0; i < ${#DATASET_NAMES[@]}; i++)) do
  dataset_name="${DATASET_NAMES[$i]}"
  rmi_config="${RMI_CONFIGS[$i]}"
  echo ">>> ${dataset_name}, config= ${rmi_config}"
  ../rmi ${DATA_PATH}/${dataset_name} rmi ${rmi_config} --use-mmap  # TODO: tune
  rm -rf ${STORAGE_PATH}/${dataset_name}_rmi_mmap
  mkdir ${STORAGE_PATH}/${dataset_name}_rmi_mmap
  mv rmi.cpp rmi.h rmi_data.h rmi_data ${STORAGE_PATH}/${dataset_name}_rmi_mmap

  ${CXX} ${CXXFLAGS} main.cpp ${STORAGE_PATH}/${dataset_name}_rmi_mmap/rmi.cpp -I ${STORAGE_PATH}/${dataset_name}_rmi_mmap -I . -o ${BINARY_PATH}/main_${dataset_name}_rmi_mmap ${CXXBACKFLAGS}
done
