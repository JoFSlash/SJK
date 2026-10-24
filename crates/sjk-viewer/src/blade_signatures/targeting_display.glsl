// Targeting Display: an old starfighter's targeting computer, as a blade. A blocky blade of chunky
// lit squares, a few of them blinking as the computer reads.
// @part Readout
// @veins Target grid
// @room 8
// @palette {"corona.rim_cool":[0.05,0.3,0.85],"corona.rim_hot":[0.2,0.8,1.0],"corona.inner":[0.75,1.0,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.2,0.75,1.0],"core.fringe_hot":[0.7,1.0,1.0],"light.color":[0.2,0.8,1.0],"trail":[0.2,0.8,1.0]}

vec3 targeting_display_glow0(vec3 g, vec2 b, float r, float px){
  // round 23: the lit squares' and the levels' edges smoothed (they stair-stepped into a jagged
  // outline); the blade keeps its blocky levels
  vec2 u = vec2(b.x*4.0, b.y*1.6), q = floor(u);
  float tk = floor(uTime*4.0);
  float mx = cellMix(u.x, px*4.0), my = cellMix(u.y, px*1.6);
  float blink = mix(mix(step(0.85, h21(q + tk)), step(0.85, h21(q + vec2(1.0, 0.0) + tk)), mx),
                    mix(step(0.85, h21(q + vec2(0.0, 1.0) + tk)), step(0.85, h21(q + vec2(1.0, 1.0) + tk)), mx), my)*part(0.0);
  float lv = lvl(g)*5.0 + 0.5;
  float step_ = (floor(lv) + smoothstep(0.85, 1.0, fract(lv)))/5.0;
  return g/max(lvl(g), 0.001)*step_*(1.0 + 0.5*blink);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = targeting_display_glow0(g, b, r, px);
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
