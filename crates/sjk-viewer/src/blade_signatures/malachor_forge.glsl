// Malachor Forge: a blade beaten out in the old Sith forges of Malachor. A blade glowing like hot
// steel; every second a hammer blow lands somewhere on it, flaring it white with a
// spray of sparks, embers fall and dark forge smoke rises in the shimmering heat. A clash
// flares the whole blade.
// @part Hammer blows
// @part Hammer sparks
// @part Forge smoke
// @room 8
// @palette {"corona.rim_cool":[0.5,0.08,0.02],"corona.rim_hot":[1.0,0.4,0.1],"corona.inner":[1.0,0.8,0.6],"core.white":[1,0.95,0.85],"core.fringe_cool":[1.0,0.35,0.08],"core.fringe_hot":[1.0,0.7,0.45],"light.color":[1.0,0.42,0.12],"trail":[1.0,0.42,0.12]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float n = floor(uTime/1.1), ph = fract(uTime/1.1)*1.1;
  float spot = exp(-pow((b.y - uLength*(0.25 + 0.6*h11(n*3.7)))/1.5, 2.0))*exp(-ph*2.0)*part(0.0);
  vec3 steel = mix(vec3(1.0, 0.45, 0.15), vec3(1.0, 0.95, 0.85), clamp(lvl(g) - 0.3, 0.0, 1.0));
  vec2 mcell;
  float crack = halo(voro(vec2(b.x*3.0, b.y*0.5), mcell), 0.05)*(0.6 + 0.4*sin(uTime*2.0 + h21(mcell)*6.0));
  return lvl(g)*vec3(1.0, 0.1, 0.05)*crack*1.2 + recol(g, steel)*(0.9 + 1.4*spot + 0.3*clamp(uSwing, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 fire = vec3(1.0, 0.55, 0.2);
  float n = floor(t/1.1), ph = fract(t/1.1);
  float yh = L*(0.25 + 0.6*h11(n*3.7));
  fx += fxSparks(ro, rd, px, yh - 1.0, yh + 1.0, 18.0*exp(-ph*3.0), 5.0, 1.1, n*1.7, fire)*part(1.0);
  fx += fxSparks(ro, rd, px, y0, y1, 6.0, 3.0, 0.7, 4.8, fire)*0.7*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.2, 2.2, 9.4, vec3(0.75, 0.45, 0.3))*part(2.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 18.0, 0.08, 3.4, vec3(1.0, 0.5, 0.2))*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.6, 1.5, 2.9, vec3(0.6, 0.12, 0.08))*0.8*part(2.0);
  return fx*uFx;
}
