// xcrun -sdk macosx metal -c src/triangle.metal

#include <metal_stdlib>

using namespace metal;

struct SceneProperties {
    float offset_x;
    float offset_y;
};

constant float2 quadVertices[] = {
    float2(-1, -1),
    float2(-1,  1),
    float2( 1,  1),
    float2(-1, -1),
    float2( 1,  1),
    float2( 1, -1)
};

struct VertexOutput {
    float4 position [[position]];
    float2 uv;
};

vertex VertexOutput vertex_main(
    device const SceneProperties& properties [[buffer(0)]],
    uint vertex_idx [[vertex_id]]
) {

    float2 position = quadVertices[vertex_idx];

    VertexOutput out;

    out.position =
        float4( position,
            0.0,
            1);

    out.uv = position * 0.5 + 0.5;
    return out;
}

fragment float4 fragment_main(
    device const SceneProperties& properties [[buffer(0)]],
    VertexOutput in [[stage_in]]) {
    float r = cos((in.uv.x) *40.0 - properties.offset_x*0.05);
    float g = cos((in.uv.y) *40.0 + properties.offset_y*0.05);
    float4 color = float4(r,g,0.5,1.0);
    return color;
}
