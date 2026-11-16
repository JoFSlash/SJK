// Flare Stick: an emergency hand flare from a starfighter's survival kit, as a blade. A red-orange
// blade that sputters and flickers like burning chemicals, spitting short sparks, with smoke
// rising off it.
// @part Sputtering sparks
// @part Flare smoke
// @room 8
// @palette {"corona.rim_cool":[0.5,0.05,0.02],"corona.rim_hot":[1.0,0.35,0.12],"corona.inner":[1.0,0.75,0.55],"core.white":[1,0.92,0.85],"core.fringe_cool":[1.0,0.3,0.1],"core.fringe_hot":[1.0,0.7,0.5],"light.color":[1.0,0.35,0.12],"trail":[1.0,0.35,0.12]}

vec3 flare_stick_glow0(vec3 g, vec2 b, float r, float px){
  float fl = 0.85 + 0.15*sin(uTime*23.0 + b.y*0.7)*sin(uTime*17.0) + 0.1*vn(vec2(b.y*0.8, uTime*9.0));
  return g*fl;
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = flare_stick_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(1.0, 0.5, 0.2);
  fx += fxSparks(ro, rd, px, y0, y1, 10.0, 2.5, 1.6, 8.8, c)*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.2, 3.0, 4.4, vec3(0.5, 0.45, 0.45))*0.6*part(1.0);
  return fx*uFx*2.40;
}
