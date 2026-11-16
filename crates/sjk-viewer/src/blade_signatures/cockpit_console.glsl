// Cockpit Console: the consoles of a starship cockpit, as a blade. Two rows of small red, green and
// amber lights blink along the pale blade as the ship's systems run, a few always on, others
// winking in turn.
// @part Console lights
// @veins Circuits
// @room 8
// @palette {"corona.rim_cool":[0.25,0.3,0.4],"corona.rim_hot":[0.75,0.85,1.0],"corona.inner":[0.92,0.96,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.65,0.75,1.0],"core.fringe_hot":[0.9,0.95,1.0],"light.color":[0.75,0.85,1.0],"trail":[0.75,0.85,1.0]}

vec3 cockpit_console_glow0(vec3 g, vec2 b, float r, float px){
  vec2 q = vec2(step(0.0, b.x), floor(b.y/1.6));
  float h = h21(q + vec2(3.1, 0.0));
  vec3 lamp = h < 0.33 ? vec3(1.0, 0.2, 0.15) : (h < 0.66 ? vec3(0.3, 1.0, 0.4) : vec3(1.0, 0.75, 0.2));
  float on = step(0.5, h11(h*91.0 + floor(uTime*(1.0 + 3.0*h))));
  float dot_ = halo(length(vec2(abs(b.x) - 0.18, (fract(b.y/1.6) - 0.5)*1.6)), 0.12);
  return g*0.9 + lvl(g)*lamp*dot_*on*1.4*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = cockpit_console_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 1.6, vec3(0.7, 0.8, 1.0))*0.15*part(0.0);
  return fx*uFx*2.40;
}
