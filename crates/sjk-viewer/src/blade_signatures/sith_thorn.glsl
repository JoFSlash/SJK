// Sith Thorn: a thorned stem grown in the dark side, as a blade. A deep crimson blade with sharp
// thorns of light jutting off both sides, and red petals of light drifting slowly down off it,
// turning as they fall; round 16 (Creyon: better for its name and rarity): dark veins pulse up
// the blade, its thorns flare in a wave as the pulse passes, and a black thorned vine coils round
// it in 3D, its thorns tipped with red light. A clash flares it.
// @part Thorns
// @part Petals
// @part Thorned vine
// @room 8
// @palette {"corona.rim_cool":[0.35,0.0,0.05],"corona.rim_hot":[0.95,0.12,0.22],"corona.inner":[0.45,0.05,0.08],"core.white":[1.0,0.7,0.72],"core.fringe_cool":[0.9,0.1,0.2],"core.fringe_hot":[1.0,0.55,0.6],"light.color":[0.9,0.1,0.2],"trail":[0.9,0.1,0.2]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  // dark veins branching through the crimson, a pulse of dark power climbing them
  vec2 cell;
  float vein = halo(voro(vec2(b.x*4.0, b.y*0.6), cell), 0.05);
  float pulse = pow(0.5 + 0.5*sin(b.y*0.25 - uTime*2.2), 6.0);
  return g*(0.9 + 0.15*bands(b.y, 0.05, 0.3))*(1.0 - 0.45*vein) + lvl(g)*vec3(1.0, 0.1, 0.15)*vein*pulse*1.4;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 red = vec3(1.0, 0.12, 0.2);
  float s, ta;
  vec3 th = vec3(0.0), pe = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) < 3.0){
    for (int i = 0; i < 9; i++){
      float fi = float(i);
      float side = mod(fi, 2.0) < 0.5 ? -1.0 : 1.0;
      float y = mix(y0, y1 - 2.0, (fi + 0.3)/9.0);
      // round 19: the thorns grow all round the stem, in three dimensions
      vec3 a = turnY(vec3(side*0.35, y, 0.0), fi*0.19);
      vec3 tip = a + turnY(vec3(side*0.9, 0.55, 0.0), fi*0.19);
      float d = raySeg(ro, rd, a, tip, s);
      float w = mix(0.14, 0.02, s);
      float wave = 1.0 + 1.5*pow(0.5 + 0.5*sin(y*0.25 - t*2.2), 6.0);
      th += (red*halo(d, max(w, px))*0.9*(1.0 - 0.5*s) + vec3(1.0, 0.6, 0.6)*halo(d, max(w*0.4, px))*0.6)*wave;
    }
  }
  if (fxNear(ro, rd, y0 - 4.0, y1) < 3.0){
    for (int i = 0; i < 12; i++){
      float fi = float(i);
      float f = fract(t*0.06*(0.7 + 0.6*h11(fi*2.7)) + h11(fi*6.1));
      // round 20 (Creyon: the petals showed as red slivers in the preview, lying in one flat sheet):
      // all round the blade in 3D, each a small oval facing the camera, turning as it falls
      vec3 p = around(0.7 + 1.5*h11(fi*3.9) + 0.4*sin(t*0.9 + fi*2.0), h11(fi*5.3) + 0.05*t, mix(y1, y0 - 4.0, f));
      float d = rayPt(ro, rd, p, ta);
      vec3 q = ro + rd*ta - p;
      vec3 fu = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
      vec2 e = rot(t*1.2 + fi)*vec2(dot(q, fu), dot(q, cross(rd, fu)));
      float pd = length(e/vec2(0.22, 0.09));
      pe += red*(halo(pd - 1.0, 0.25)*0.8 + halo(d, 0.5)*0.08)*smoothstep(0.0, 0.1, f)*(1.0 - smoothstep(0.85, 1.0, f));
    }
  }
  fx += th*part(0.0) + pe*part(1.0);
  return fx*uFx;
}

// round 16: a black thorned vine coiling up round the blade, its thorns tipped with red light;
// it circles the blade, so it is drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s, ta;
  float y0 = L*0.12, y1 = L*0.9;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) > 3.0) return fx;
  float best = 1e3;
  vec3 prev = vec3(0.0);
  for (int i = 0; i <= 24; i++){
    float f = float(i)/24.0;
    vec3 p = around(1.3 + 0.1*sin(f*20.0), f*2.2 + t*0.03, mix(y0, y1, f));
    if (i > 0) best = min(best, raySeg(ro, rd, prev, p, s));
    prev = p;
  }
  fx += vec3(0.6, 0.02, 0.06)*halo(best, 0.2)*0.9 + vec3(1.0, 0.3, 0.35)*halo(best, max(0.04, px))*0.8;
  for (int i = 0; i < 9; i++){
    float f = (float(i) + 0.5)/9.0;
    vec3 p = around(1.3, f*2.2 + t*0.03, mix(y0, y1, f));
    vec3 out_ = normalize(vec3(p.x, 0.0, p.z));
    float pulse = 0.5 + 0.5*pow(0.5 + 0.5*sin(p.y*0.25 - t*2.2), 6.0)*2.0;
    float d = raySeg(ro, rd, p, p + out_*0.7 + vec3(0.0, 0.25, 0.0), s);
    fx += vec3(1.0, 0.15, 0.2)*halo(d, max(mix(0.06, 0.015, s), px))*(0.5 + s)*pulse;
  }
  return fx*uFx;
}
