// Ahch-To Glow: the glowing tide pools of Ahch-To as a blade. Soft pulses of sea light climb a
// sea-green blade, specks of plankton light drift round it and a faint sea mist rises off it. A
// swing stirs up more plankton; a clash sends a ring of sea light.
// @part Plankton glow
// @room 8
// @palette {"corona.rim_cool":[0.0,0.18,0.4],"corona.rim_hot":[0.1,0.85,0.8],"corona.inner":[0.55,1.0,0.92],"core.white":[0.85,1.0,0.97],"core.fringe_cool":[0.1,0.75,0.85],"core.fringe_hot":[0.6,1.0,0.92],"light.color":[0.1,0.85,0.8],"trail":[0.12,0.8,0.8]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float p = pow(0.5 + 0.5*sin(b.y*0.5 - uTime*1.6), 4.0);
  return g*(0.85 + 0.6*p);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.3, 1.0, 0.9);
  fx += fxMotes(ro, rd, px, y0, y1, 0.7, 3.2, 14.0 + 12.0*sw, 0.03, 1.7, c)*(0.7 + 0.8*sw);
  fx += fxAura(ro, rd, y0, y1, 1.8, c)*0.5;
  return fx*uFx*part(0.0);
}
