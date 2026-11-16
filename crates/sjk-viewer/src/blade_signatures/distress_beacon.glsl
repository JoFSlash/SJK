// Distress Beacon: the beacon of a crashed ship calling for help, as a blade. The blade blinks out
// its call in short and long flashes, over and over, and puffs of orange signal smoke rise off it.
// @part Distress signal
// @part Beacon smoke
// @embers Sparks
// @room 8
// @palette {"corona.rim_cool":[0.5,0.1,0.02],"corona.rim_hot":[1.0,0.45,0.15],"corona.inner":[1.0,0.8,0.6],"core.white":[1,0.94,0.88],"core.fringe_cool":[1.0,0.4,0.12],"core.fringe_hot":[1.0,0.75,0.55],"light.color":[1.0,0.45,0.15],"trail":[1.0,0.45,0.15]}

vec3 distress_beacon_glow0(vec3 g, vec2 b, float r, float px){
  float s = fract(uTime/4.0)*16.0;
  float on = step(s, 1.0) + step(2.0, s)*step(s, 3.0) + step(4.0, s)*step(s, 7.0) + step(8.0, s)*step(s, 11.0);
  return g*(0.7 + 0.6*clamp(on, 0.0, 1.0)*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = distress_beacon_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.6, 4.0, 7.3, vec3(1.0, 0.5, 0.2))*0.6*part(1.0);
  return fx*uFx*2.40;
}
