// Code Cylinder: an Imperial officer's code cylinder, as a blade. Data stripes of different widths
// scroll slowly up the blade, brighter and dimmer.
// @part Code stripes
// @veins Data traces
// @room 8
// @palette {"corona.rim_cool":[0.6,0.62,0.66],"corona.rim_hot":[0.95,0.95,0.98],"corona.inner":[1.0,1.0,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.85,0.85,0.9],"core.fringe_hot":[1.0,1.0,1.0],"light.color":[0.9,0.9,0.95],"trail":[0.9,0.9,0.95]}

vec3 code_cylinder_glow0(vec3 g, vec2 b, float r, float px){
  // round 23: each stripe's edge smoothed over a pixel (it stair-stepped)
  float u = b.y*1.2 - uTime*0.8, k = floor(u);
  float s = mix(step(0.45, h11(k*3.7)), step(0.45, h11((k + 1.0)*3.7)), cellMix(u, px*1.2));
  return g*(0.8 + 0.3*s*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = code_cylinder_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += vec3(0.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*2.00;
  return vec3(0.0);
}
