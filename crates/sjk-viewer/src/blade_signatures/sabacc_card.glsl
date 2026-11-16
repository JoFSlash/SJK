// Sabacc Card: the shimmering face of a sabacc card, as a blade. Rainbow colours shift slowly
// along the blade, as on a card turned in the light.
// @part Shimmer
// @motes Glints
// @room 8
// @palette {"corona.rim_cool":[0.45,0.3,0.8],"corona.rim_hot":[0.6,0.9,1.0],"corona.inner":[1.0,0.95,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.7,0.5,1.0],"core.fringe_hot":[0.8,1.0,0.9],"light.color":[0.7,0.6,1.0],"trail":[0.65,0.6,1.0]}

vec3 sabacc_card_glow0(vec3 g, vec2 b, float r, float px){
  vec3 c = hsv(fract(b.y*0.03 + b.x*0.4 - uTime*0.08), 0.5, 1.0);
  return mix(g, recol(g, c), 0.75*part(0.0) + 0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = sabacc_card_glow0(g, b, r, px);
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
