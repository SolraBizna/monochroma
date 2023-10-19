/* -*- glsl -*- */

in mediump vec2 pos;
in mediump vec2 vert_uv;

out mediump vec2 frag_uv;

void main() {
  gl_Position = vec4(pos,0.0,1.0);
  frag_uv = vert_uv;
}
