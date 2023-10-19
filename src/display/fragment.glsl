/* -*- glsl -*- */

uniform usampler2D bits;
uniform mediump vec4 zerocolor;
uniform mediump vec4 onecolor;
in mediump vec2 frag_uv;
out mediump vec4 result;

void main() {
    ivec2 frag_uvi = ivec2(floor(frag_uv));
    int bit = (frag_uvi.x % 32) ^ 31;
    uint samp = texelFetch(bits, ivec2(frag_uvi.x / 32, frag_uvi.y), 0).r;
    if(((samp >> bit) & 1u) == 0u) {
        result = zerocolor;
    } else {
        result = onecolor;
    }
}
