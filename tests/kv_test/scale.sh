set -e
echo "Training with compiler flags ${CXX} ${CXXFLAGS} ${CXXBACKFLAGS}"

ROOT=$1
DATA_PATH=$2
STORAGE_PATH="${ROOT}/storage"
BINARY_PATH="${ROOT}/binary"

mkdir -p ${STORAGE_PATH}
mkdir -p ${BINARY_PATH}

DATASET_NAMES=(
 "gmm_k100_200M_uint64"
 "gmm_k100_400M_uint64"
 "gmm_k100_600M_uint64"
 "gmm_k100_800M_uint64"
)

RMI_CONFIGS=( # configurations obtained from rmi optimizer
 "linear_spline,linear  1024"
 "linear_spline,linear  1048576"
 "linear_spline,linear  1024"
 "linear_spline,linear  1048576"
)

echo "Training mmap"
for ((i = 0; i < ${#DATASET_NAMES[@]}; i++)) do
  dataset_name="${DATASET_NAMES[$i]}"
  rmi_config="${RMI_CONFIGS[$i]}"
  echo ">>> ${dataset_name}, config= ${rmi_config}"
  SECONDS=0
  ../rmi ${DATA_PATH}/${dataset_name} rmi ${rmi_config} --use-mmap  
  rm -rf ${STORAGE_PATH}/${dataset_name}_rmi_mmap
  mkdir ${STORAGE_PATH}/${dataset_name}_rmi_mmap
  mv rmi.cpp rmi.h rmi_data.h rmi_data ${STORAGE_PATH}/${dataset_name}_rmi_mmap

  ${CXX} ${CXXFLAGS} main.cpp ${STORAGE_PATH}/${dataset_name}_rmi_mmap/rmi.cpp -I ${STORAGE_PATH}/${dataset_name}_rmi_mmap -I . -o ${BINARY_PATH}/main_${dataset_name}_rmi_mmap ${CXXBACKFLAGS}

  DURATION=$SECONDS
  echo "-------TIME---------"
  echo "It takes $(($DURATION * 1000)) milliseconds to build ${dataset_name} with max threads"
  echo "-------TIME---------"
done
