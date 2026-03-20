Texture2D    Texture : register(t0, space2);
SamplerState Sampler : register(s0, space2);

struct Input {
    float2 TexCoord : TEXCOORD0;
    float4 Color    : TEXCOORD1;
};

float4 main(Input i) : SV_Target0 {
    return i.Color * Texture.Sample(Sampler, i.TexCoord);
}