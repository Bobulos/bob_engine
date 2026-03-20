struct SpriteData {
    float3 Position;
    float  Rotation;
    float2 Scale;
    float2 Padding;        // std140 alignment float3 must pad to 16 bytes
    float  TexU, TexV, TexW, TexH;
    float4 Color;
};

StructuredBuffer<SpriteData> DataBuffer : register(t0, space0);

cbuffer Camera : register(b0, space1) {
    float4x4 ViewProjection;
};

struct Output {
    float2 Texcoord : TEXCOORD0;
    float4 Color    : TEXCOORD1;
    float4 Position : SV_Position;
};

static const uint  triIndices[6]  = { 0, 1, 2, 3, 2, 1 };
static const float2 quadVerts[4]  = {
    {0.0f, 0.0f}, {1.0f, 0.0f},
    {0.0f, 1.0f}, {1.0f, 1.0f}
};

Output main(uint id : SV_VertexID) {
    uint vert   = triIndices[id % 6];
    uint sprite = id / 6;
    SpriteData s = DataBuffer[sprite];

    float2 uv[4] = {
        {s.TexU,          s.TexV         },
        {s.TexU + s.TexW, s.TexV         },
        {s.TexU,          s.TexV + s.TexH},
        {s.TexU + s.TexW, s.TexV + s.TexH}
    };

    float c = cos(s.Rotation), sn = sin(s.Rotation);
    float2 coord = quadVerts[vert] * s.Scale;
    coord = mul(coord, float2x2(c, sn, -sn, c));

    Output o;
    o.Position = mul(ViewProjection, float4(coord + s.Position.xy, s.Position.z, 1.0));
    o.Texcoord = uv[vert];
    o.Color    = s.Color;
    return o;
}