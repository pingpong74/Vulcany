// module vulkany;

[[vk::binding(0, 0)]]
StructuredBuffer<uint64_t> device_address_buffer;

namespace vulcany {
__generic<T> T *get_buffer(uint32_t index) {
  return (T *)device_address_buffer[index];
}
}
