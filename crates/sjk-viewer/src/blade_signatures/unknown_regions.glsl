// Unknown Regions: the uncharted dark beyond the galaxy's edge. A deep violet blade with a shadowed
// heart, nebula swirling slowly inside it and round it in soft violet clouds, faint stars drifting
// through them and a far spiral galaxy turning slowly beside it (round 15: it replaced threads of
// starlight it shared with the Aurodium). A clash flares it.
// @part Nebula
// @part Drifting stars
// @part Hyperspace storm
// @room 8
// @palette {"corona.rim_cool":[0.15,0.05,0.4],"corona.rim_hot":[0.6,0.3,1.0],"corona.inner":[0.12,0.04,0.3],"core.white":[0.45,0.3,0.8],"core.fringe_cool":[0.35,0.15,0.9],"core.fringe_hot":[0.7,0.5,1.0],"light.color":[0.5,0.3,1.0],"trail":[0.5,0.3,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float n = fbm3(vec2(b.x*1.2 + uTime*0.1, b.y*0.25 - uTime*0.15));
  float heart = 1.0 - 0.4*exp(-pow(b.x/0.25, 2.0));
  return g*heart*(0.6 + 0.8*n);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 v = vec3(0.5, 0.3, 1.0), w = vec3(0.85, 0.85, 1.0);
  fx += fxSmoke(ro, rd, y0, y1, 4, 0.6, 8.8, v)*1.1*part(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 4.5, 28.0, 0.015, 3.9, w)*0.8*part(1.0);
  return fx*uFx;
}
// round 16: the hyperspace storm: three streams of glowing gas whirling round the blade on tilted
// paths, each a long soft ribbon fading behind its head, sparks of light caught in them; they
// circle the blade, so they are drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, L*0.15, L*0.95) > 6.0) return fx;
  for (int j = 0; j < 3; j++){
    float fj = float(j);
    float best = 1e3, bs = 0.0;
    vec3 prev = vec3(0.0);
    for (int i = 0; i <= 14; i++){
      float tt = t*0.8 - float(i)*0.09 + fj*2.1;
      float a = tt*0.14 + fj/3.0;
      vec3 p = around(2.2 + 0.9*sin(tt*0.5 + fj), a, L*(0.55 + 0.3*sin(tt*0.23 + fj*1.7)) + 2.0*sin(a*6.2832 + fj));
      if (i > 0){ float d = raySeg(ro, rd, prev, p, s); if (d < best){ best = d; bs = (float(i) - 1.0 + s)/14.0; } }
      prev = p;
    }
    float tail = 1.0 - bs;
    vec3 c = j == 1 ? vec3(0.35, 0.5, 1.0) : vec3(0.6, 0.3, 1.0);
    float spark = step(0.8, h11(floor(bs*20.0) + floor(t*4.0) + fj*9.0));
    fx += (c*halo(best, 0.25 + 0.5*bs)*tail*0.9 + vec3(0.9, 0.85, 1.0)*halo(best, max(0.05, px))*(pow(tail, 5.0)*1.5 + spark*tail));
  }
  return fx*uFx;
}
