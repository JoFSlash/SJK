// Power Coupling: a starship's power coupling, as a blade. A glowing copper filament winds through
// the blade, brighter where the current runs, and now and then a spark jumps off it.
// @part Sparks
// @embers Sparks
// @room 8
// @palette {"corona.rim_cool":[0.45,0.16,0.04],"corona.rim_hot":[1.0,0.48,0.12],"corona.inner":[1.0,0.78,0.5],"core.white":[1.0,0.9,0.75],"core.fringe_cool":[0.9,0.4,0.1],"core.fringe_hot":[1.0,0.7,0.35],"light.color":[1.0,0.5,0.15],"trail":[1.0,0.5,0.18]}

vec3 power_coupling_glow0(vec3 g, vec2 b, float r, float px){
  float c = halo(abs(b.x - 0.28*sin(b.y*1.6)), 0.05)*(0.6 + 0.4*sin(b.y*0.4 - uTime*3.0));
  return g*0.85 + lvl(g)*vec3(1.0, 0.6, 0.3)*c;
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = power_coupling_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSparks(ro, rd, px, y0, y1, 3.0, 2.0, 0.7, 6.4, vec3(1.0, 0.7, 0.4))*part(0.0);
  return fx*uFx*2.00;
}
