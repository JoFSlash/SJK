// Bespin Storm: a storm in the clouds round Cloud City. A blade with wind bands slanting across it
// in the cloud's colour, soft sunset clouds of rose and gold billowing round it, gale streaks
// racing past (round 15: no lightning, which it shared with the other storm blades). Round 19
// (Creyon: the effect looked cheap on it): the gas giant's banded storm turns round it in 3D,
// layers of sunset cloud swirling at their own speeds, and shimmering tibanna gas rises up it in
// slow rings and glints. A clash flares it.
// @part Cloud City clouds
// @part Gale
// @part Tibanna gas
// @room 8
// @palette {"corona.rim_cool":[0.55,0.35,0.25],"corona.rim_hot":[1.0,0.75,0.55],"corona.inner":[1.0,0.93,0.85],"core.white":[1,1,1],"core.fringe_cool":[1.0,0.65,0.45],"core.fringe_hot":[1.0,0.88,0.75],"light.color":[1.0,0.75,0.55],"trail":[1.0,0.75,0.55]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g*(0.8 + 0.35*bands(b.y + b.x*4.0, 0.15, 1.5));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// round 19: the gas giant's storm bands: a thick shell of sunset cloud round the blade, banded
// across and swirling round it (faster near it), lit gold above and rose below; a cloud in
// three dimensions, the ray marched through it
float bs_cloud(vec3 p, float y0, float y1, out float tone){
  float r = length(p.xz);
  float shell = exp(-pow((r - 2.9)/1.5, 2.0));
  vec2 q = rot(uTime*0.22 + 1.6/(r + 0.8))*p.xz;
  float n = fbm3(vec2(q.x*0.32 + p.y*0.05, p.y*0.26 + q.y*0.3));
  float bands = 0.5 + 0.5*sin(p.y*0.85 + 2.4*n + q.x*0.15);
  tone = bands;
  float win = smoothstep(y0 - 2.0, y0 + 3.0, p.y)*(1.0 - smoothstep(y1 - 5.0, y1, p.y));
  return smoothstep(0.38, 0.85, n*0.75 + bands*0.4)*shell*win;
}
vec3 bs_bands(vec3 ro, vec3 rd, float y0, float y1){
  vec2 span = raySpan(ro, rd, 5.8, y0 - 2.0, y1);
  if (span.x >= span.y) return vec3(0.0);
  float dt = (span.y - span.x)/7.0;
  vec3 acc = vec3(0.0);
  for (int i = 0; i < 7; i++){
    float tone;
    float d = bs_cloud(ro + rd*(span.x + (float(i) + 0.5)*dt), y0, y1, tone);
    acc += mix(vec3(0.95, 0.5, 0.42), vec3(1.0, 0.86, 0.56), tone)*d;
  }
  return acc*dt/(1.77*1.5)*0.45*(1.0 + 0.8*clamp(uClash, 0.0, 1.0));
}
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.08, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 rose = vec3(1.0, 0.65, 0.5), gold = vec3(1.0, 0.82, 0.55);
  fx += bs_bands(ro, rd, y0, y1)*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 4.2, 1.2, 1.1, rose)*0.55*part(0.0);
  fx += fxSmoke(ro, rd, y0 + 4.0, y1, 3, 0.8, 7.5, gold)*0.35*part(0.0);
  float s;
  if (fxNear(ro, rd, y0, y1) < 7.0){
    for (int i = 0; i < 16; i++){
      float fi = float(i);
      float f = fract(t*0.5*(0.7 + 0.6*h11(fi*2.3)) + h11(fi*5.9));
      float y = mix(y0, y1, h11(fi*1.7 + floor(t*0.5 + h11(fi*5.9))));
      float x = mix(-5.0, 5.0, f);
      // round 19: each gust at its own depth, racing past in three dimensions
      float z = (h11(fi*4.1 + floor(t*0.5 + h11(fi*5.9))) - 0.5)*7.0;
      float d = raySeg(ro, rd, vec3(x, y, z), vec3(x + 1.8 + 1.2*sw, y - 0.3, z + 0.3), s);
      fx += vec3(1.0, 0.95, 0.9)*(halo(d, max(0.03, px))*0.6 + halo(d, 0.2)*0.1)*sin(3.1416*f)*part(1.0);
    }
  }
  // tibanna gas: slow shimmering rings rising up the blade and pale gold glints
  fx += fxRingWaves(ro, rd, px, y0, y1, 3.0, 2.2, 3.0, 0.31, vec3(1.0, 0.9, 0.7))*0.55*part(2.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.2, 22.0, 0.05, 6.6, vec3(1.0, 0.92, 0.75))*0.8*part(2.0);
  return fx*uFx;
}
