// Mygeeto Frost: the frozen crystal world of Mygeeto, as a blade. A frosty blade, pale blue near
// the hilt and white above, with a drop of melt falling off it now and then.
// @part Melt drops
// @motes Ice glints
// @room 8
// @palette {"corona.rim_cool":[0.45,0.15,0.55],"corona.rim_hot":[0.7,0.6,1.0],"corona.inner":[0.92,0.92,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.6,0.5,1.0],"core.fringe_hot":[0.9,0.85,1.0],"light.color":[0.4,0.55,1.0],"trail":[0.45,0.55,1.0]}

vec3 mygeeto_frost_glow0(vec3 g, vec2 b, float r, float px){
  float m = smoothstep(uLength*0.3, uLength*0.6, b.y);
  return recol(g, mix(vec3(0.55, 0.8, 1.0), vec3(0.95, 0.98, 1.0), m))*(0.85 + 0.1*fbm3(vec2(b.x*5.0, b.y)));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = mygeeto_frost_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 1.0, 2.0, 5.0, 0.35, 0.0, 3.1, vec3(0.8, 0.92, 1.0))*part(0.0);
  return fx*uFx*2.00;
}
