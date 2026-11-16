// Cargo Stack: a freighter's hold being loaded, as a blade. Blocks of light stack up the blade from
// the hilt one after another, flash when the stack reaches the top and clear, and a few more crates
// of light drop past beside it.
// @part Stacking cargo
// @part Falling cargo
// @motes Hold lights
// @room 8
// @palette {"corona.rim_cool":[0.1,0.15,0.55],"corona.rim_hot":[0.3,0.6,1.0],"corona.inner":[0.7,0.85,1.0],"core.white":[0.9,0.95,1.0],"core.fringe_cool":[0.3,0.55,1.0],"core.fringe_hot":[0.8,0.9,1.0],"light.color":[0.35,0.65,1.0],"trail":[0.35,0.65,1.0]}

vec3 cargo_stack_glow0(vec3 g, vec2 b, float r, float px){
  float cyc = fract(uTime/8.0);
  float n = cyc*(uLength/2.5 + 2.0);
  // round 23: the top of the stack smoothed over a pixel (it stair-stepped)
  float k = floor(b.y/2.5);
  float lit = mix(step(k + 1.0, n), step(k + 2.0, n), cellMix(b.y/2.5, px/2.5))*part(0.0);
  float edge = halo((0.5 - abs(fract(b.y/2.5) - 0.5))*2.5, 0.05);
  float clear = smoothstep(0.92, 1.0, cyc)*part(0.0);
  return g*(0.6 + 0.45*lit + 0.8*clear + 0.4*(1.0 - part(0.0))) + lvl(g)*vec3(1.0)*edge*0.3*lit;
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = cargo_stack_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 1.8, 6.0, 9.0, 0.9, 0.0, 3.3, vec3(0.9, 0.95, 1.0))*0.6*part(1.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*2.40;
  return vec3(0.0);
}
