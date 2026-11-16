// Pasaana Stone: the layered desert rock of Pasaana, as a blade. Warm bands of stone with light
// glowing between the layers, and a little sand trickling off it.
// @part Sand trickle
// @wisps Dust
// @room 8
// @palette {"corona.rim_cool":[0.55,0.25,0.1],"corona.rim_hot":[1.0,0.58,0.25],"corona.inner":[1.0,0.82,0.6],"core.white":[1.0,0.92,0.8],"core.fringe_cool":[0.9,0.45,0.2],"core.fringe_hot":[1.0,0.75,0.5],"light.color":[1.0,0.55,0.25],"trail":[0.95,0.55,0.28]}

vec3 pasaana_stone_glow0(vec3 g, vec2 b, float r, float px){
  float layer = sin(b.y*1.3 + 0.6*sin(b.y*0.3) + b.x*2.0);
  vec3 c = mix(vec3(0.85, 0.55, 0.3), vec3(1.0, 0.8, 0.55), 0.5 + 0.5*layer);
  return recol(g, c)*(0.75 + 0.35*halo(abs(layer), 0.15));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = pasaana_stone_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 1.2, 5.0, 5.0, 0.5, 0.0, 6.9, vec3(1.0, 0.8, 0.55))*0.5*part(0.0);
  return fx*uFx*2.00;
}
