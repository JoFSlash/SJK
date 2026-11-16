// Frosted Viewport: a frosted viewport on an ice world, as a blade. The blade's light is soft and
// milky, as if seen through frost, and a little cold mist sinks off it.
// @part Frost mist
// @motes Frost glints
// @room 8
// @palette {"corona.rim_cool":[0.3,0.45,0.85],"corona.rim_hot":[0.7,0.8,1.0],"corona.inner":[0.95,0.97,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.6,0.7,1.0],"core.fringe_hot":[0.9,0.94,1.0],"light.color":[0.55,0.68,1.0],"trail":[0.55,0.68,1.0]}

vec3 frosted_viewport_glow0(vec3 g, vec2 b, float r, float px){
  float fr = fbm3(vec2(b.x*4.0, b.y*0.8));
  return mix(g, recol(g, vec3(0.92, 0.96, 1.0)), 0.4)*(0.8 + 0.2*fr);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = frosted_viewport_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.0, -1.5, 2.1, vec3(0.7, 0.8, 0.95))*0.4*part(0.0);
  return fx*uFx*2.00;
}
