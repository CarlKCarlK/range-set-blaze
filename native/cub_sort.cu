#include <cstddef>
#include <cstdint>

#include <cuda_runtime_api.h>
#include <cub/device/device_radix_sort.cuh>

extern "C" int rsb_cub_sort_keys_u32(
    std::uint64_t temp_storage,
    std::size_t* temp_storage_bytes,
    std::uint64_t keys_in,
    std::uint64_t keys_out,
    std::size_t num_items,
    std::uint64_t stream,
    std::uint64_t* sorted_keys) {
  cub::DoubleBuffer<std::uint32_t> keys(
      reinterpret_cast<std::uint32_t*>(keys_in),
      reinterpret_cast<std::uint32_t*>(keys_out));
  const cudaError_t status = cub::DeviceRadixSort::SortKeys(
      reinterpret_cast<void*>(temp_storage), *temp_storage_bytes, keys,
      num_items, 0, 32, reinterpret_cast<cudaStream_t>(stream));
  if (status == cudaSuccess && sorted_keys != nullptr) {
    *sorted_keys = reinterpret_cast<std::uint64_t>(keys.Current());
  }
  return static_cast<int>(status);
}
