
struct VertexInput {
  [[vk::location(0)]]
  float2 pos;
  [[vk::location(1)]]
  float3 color;
};

struct VertexOutput {
  float4 pos : SV_POSITION;
  [[vk::location(0)]]
  float3 color;
};

[shader("vertex")]
VertexOutput main(VertexInput vsin) {
  VertexOutput output;
  output.pos = float4(vsin.pos.xy, 0, 0);
  output.color = vsin.color;

  return output;
}
