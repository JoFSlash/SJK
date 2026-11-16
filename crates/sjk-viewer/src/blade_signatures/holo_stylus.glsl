// Holo-Stylus: a stroke of a bright holo-stylus, as a blade. The blade is drawn in streaky, uneven
// strokes, and a brighter stroke runs up it again and again.
// @part Stylus stroke
// @motes Stylus sparks
// @room 8
// @palette {"corona.rim_cool":[0.65,0.75,0.0],"corona.rim_hot":[0.95,1.0,0.2],"corona.inner":[1.0,1.0,0.6],"core.white":[1.0,1.0,0.8],"core.fringe_cool":[0.8,0.95,0.1],"core.fringe_hot":[1.0,1.0,0.5],"light.color":[0.9,1.0,0.15],"trail":[0.9,1.0,0.15]}

vec3 holo_stylus_glow0(vec3 g, vec2 b, float r, float px){
  float streak = 0.8 + 0.2*sin(b.x*30.0 + 2.0*vn(vec2(b.y*0.3, 1.0)));
  float head_ = halo(b.y - mod(uTime*15.0, uLength + 15.0), 3.0);
  return g*streak*(0.9 + 0.5*head_*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = holo_stylus_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += vec3(0.0);
  return fx*uFx*2.00;
}
