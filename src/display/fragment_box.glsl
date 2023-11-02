/* -*- glsl -*- */

uniform usampler2D bits;
uniform mediump vec4 zerocolor;
uniform mediump vec4 onecolor;
in mediump vec2 frag_uv;
out mediump vec4 result;

float retrieve_bit(ivec2 frag_uvi) {
    int bit = (frag_uvi.x % 32) ^ 31;
    uint samp = texelFetch(bits, ivec2(frag_uvi.x / 32, frag_uvi.y), 0).r;
    if(((samp >> bit) & 1u) == 0u) {
        return 0.0;
    } else {
        return 1.0;
    }
}

float calculate_coverage(float coordinate, float delta) {
    float fract_coordinate = fract(coordinate);
    return clamp((fract_coordinate - 1.0 + delta) / delta, 0.0, 1.0);
}

void main() {
    ivec2 top_left = ivec2(floor(frag_uv));
    float x_coverage = calculate_coverage(frag_uv.x, dFdx(frag_uv.x));
    float y_coverage = calculate_coverage(frag_uv.y, -dFdy(frag_uv.y));
    float top_pix = mix(retrieve_bit(top_left), retrieve_bit(top_left + ivec2(1,0)), x_coverage);
    float bot_pix = mix(retrieve_bit(top_left+ivec2(0,1)), retrieve_bit(top_left + ivec2(1,1)), x_coverage);
    float pix = mix(top_pix, bot_pix, y_coverage);
    result = mix(zerocolor, onecolor, pix);
}
