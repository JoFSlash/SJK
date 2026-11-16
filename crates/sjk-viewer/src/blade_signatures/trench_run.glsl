// Trench Run: the run down the battle station's trench, as a blade. Grey metal panels race up the
// pale blade with dark seams between them, and small lights flash past on their edges.
// @part Trench lights
// @embers Turbolaser sparks
// @room 8
// @palette {"corona.rim_cool":[0.25,0.28,0.35],"corona.rim_hot":[0.75,0.8,0.9],"corona.inner":[0.92,0.94,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.65,0.7,0.85],"core.fringe_hot":[0.9,0.92,1.0],"light.color":[0.75,0.8,0.95],"trail":[0.75,0.8,0.95]}

vec3 trench_run_glow0(vec3 g, vec2 b, float r, float px){
  float y = b.y - uTime*12.0;
  float k = floor(y/3.0);
  float seam = halo((0.5 - abs(fract(y/3.0) - 0.5))*3.0, 0.08);
  float panel = 0.8 + 0.2*h11(k*2.7);
  float lamp = halo(length(vec2(abs(b.x) - 0.25, (fract(y/3.0) - 0.15)*3.0)), 0.12)*step(0.5, h11(k*5.3));
  return g*panel*(1.0 - 0.35*seam) + lvl(g)*vec3(1.0, 0.6, 0.3)*lamp*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = trench_run_glow0(g, b, r, px);
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
