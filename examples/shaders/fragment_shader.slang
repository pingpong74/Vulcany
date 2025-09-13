struct VertexOutput {
  [[vk::binding(0)]]
  float3 color;
};

[shader("fragment")]
float4 main(VertexOutput input) { return float4(input.color, 0); }
