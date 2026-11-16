// Corellian Pearl: a pearl from the seas of Corellia, as a blade. A smooth pale blade with a soft
// pink and blue sheen moving slowly over it.
// @part Sheen
// @motes Glints
// @room 8
// @palette {"corona.rim_cool":[0.7,0.62,0.8],"corona.rim_hot":[1.0,0.88,0.92],"corona.inner":[1.0,0.97,0.96],"core.white":[1.0,0.98,0.96],"core.fringe_cool":[0.85,0.8,0.95],"core.fringe_hot":[1.0,0.92,0.94],"light.color":[0.98,0.88,0.92],"trail":[0.95,0.88,0.94]}

vec3 corellian_pearl_glow0(vec3 g, vec2 b, float r, float px){
  vec3 sheen = mix(vec3(1.0, 0.85, 0.92), vec3(0.85, 0.92, 1.0), 0.5 + 0.5*sin(b.y*0.2 + b.x*3.0 - uTime*0.5));
  return mix(recol(g, vec3(0.97, 0.95, 0.95)), recol(g, sheen), part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = corellian_pearl_glow0(g, b, r, px);
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
