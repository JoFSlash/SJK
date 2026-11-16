// Kessel Spice: a pale gold blade glittering with refined spice from the mines of Kessel. Fine
// crystals of spice sparkle inside its glow and drift off it as glittering dust that hangs and
// slowly falls, catching the light; now and then a shimmering haze of spice vapour curls up
// round it. A swing throws a stream of glittering dust off the blade; a clash bursts a cloud
// of sparkling spice.
// @part Spice glitter
// @room 6
// @palette {"corona.rim_cool":[0.55,0.38,0.10],"corona.rim_hot":[1.0,0.80,0.40],"corona.inner":[1.0,0.95,0.80],"core.white":[1.0,0.98,0.92],"core.fringe_cool":[1.0,0.75,0.35],"core.fringe_hot":[1.0,0.92,0.70],"light.color":[1.0,0.82,0.45],"trail":[1.0,0.85,0.5]}

float ks_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
vec3 base_vol(vec3 ro, vec3 rd, float px){
  float t = uTime, top = ks_top(), s, ta;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 gold = vec3(1.0, 0.8, 0.4), white = vec3(1.0, 0.97, 0.9);
  vec3 col = vec3(0.0);
  if (top < 0.5) return col;
  // glittering dust drifting off the blade and slowly falling
  for (int i = 0; i < 28; i++){
    float fi = float(i);
    float f = fract(t*(0.12 + 0.08*h11(fi*2.1)) + h11(fi*6.7));
    float k = floor(t*(0.12 + 0.08*h11(fi*2.1)) + h11(fi*6.7));
    float y0 = (0.05 + 0.9*h11(fi*3.3 + k))*top;
    vec3 p = around(0.5 + 2.6*sqrt(f), h11(fi*9.1 + k) + 0.1*f, y0 - 1.5*f*f);
    float d = rayPt(ro, rd, p, ta);
    float tw = pow(0.5 + 0.5*sin(t*(8.0 + 6.0*h11(fi)) + fi*3.0), 6.0);
    col += mix(gold, white, tw)*(halo(d, max(0.04, px))*(0.6 + 1.6*tw) + halo(d, 0.18)*0.1)*sin(3.1416*f);
  }
  // spice vapour: a few shimmering wisps curling up round it
  for (int i = 0; i < 6; i++){
    float fi = float(i);
    float f = fract(t*0.18 + fi/6.0);
    vec3 p = around(0.8 + 0.8*f, fi*0.37 + f*0.4, (0.15 + 0.7*h11(fi*4.4 + floor(t*0.18 + fi/6.0)))*top + 2.5*f);
    float d = rayPt(ro, rd, p, ta);
    col += gold*halo(d, 0.5 + 0.5*f)*0.18*sin(3.1416*f);
  }
  // a swing throws a stream of dust, a clash a sparkling cloud
  if (sw + cl > 0.01){
    for (int i = 0; i < 16; i++){
      float fi = float(i);
      vec3 p = around(0.8 + 3.0*h11(fi*1.7), h11(fi*5.3) + t*0.3, (0.2 + 0.7*h11(fi*2.9))*top);
      float d = rayPt(ro, rd, p, ta);
      col += white*halo(d, max(0.05, px))*1.6*(sw*0.7 + cl)*(0.5 + 0.5*sin(t*20.0 + fi));
    }
  }
  return col*uFx;
}
// the blade: Sol's glow and core from the file, spice crystals sparkling inside it
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec2 q = vec2(b.x*3.0, b.y*1.2 - uTime*0.3);
  vec2 c = floor(q);
  float sp = step(0.82, h21(c))*fillD(length(fract(q) - h22(c)) - 0.08, px*3.0);
  // round 19 (a cut-off check: crystals were drawn out to the quad's edge and cut there): only
  // inside its glow
  sp *= 1.0 - smoothstep(1.2, 2.6, abs(b.x));
  float tw = pow(0.5 + 0.5*sin(uTime*7.0 + h21(c)*40.0), 4.0);
  return g*(0.95 + 0.4*clamp(uClash, 0.0, 1.0)) + vec3(1.0, 0.95, 0.8)*sp*tw*1.5*smoothstep(0.0, 0.5, b.y)*(1.0 - smoothstep(uLength - 0.5, uLength, b.y));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// round 11: sig_vol gated by part 0
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  return base_vol(ro, rd, px);
}
