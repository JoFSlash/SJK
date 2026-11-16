// Kyber Cluster: a cluster of raw kyber crystals bonded into one blade. A clear white-blue blade
// with crystal facets glinting in it; round it, a slow ring of kyber shards floats in 3D, each
// glowing its own colour, linked to the blade by a thin beam of light that pulses as the crystals
// sing to it; now and then one flares and sends a bead of its colour racing into the blade. A
// swing spins the ring; a clash makes every crystal flash at once. A chroma: the blade takes its
// wearer's colour, the crystals and their song keep their own.
// @part Kyber crystals
// @part Crystal song
// @room 7
// @solid 1
// @palette {"corona.rim_cool":[0.20,0.30,0.70],"corona.rim_hot":[0.60,0.80,1.0],"corona.inner":[0.88,0.95,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.50,0.70,1.0],"core.fringe_hot":[0.85,0.95,1.0],"light.color":[0.65,0.80,1.0],"trail":[0.70,0.85,1.0]}

float kc_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
vec3 kc_col(float i){ return hsv(fract(i*0.137 + 0.55), 0.65, 1.0); }
vec3 kc_pos(float i, out vec3 axis){
  float a = i/9.0 + uTime*(0.04);
  float y = uLength*(0.15 + 0.08*i) + 0.6*sin(uTime*0.5 + i*1.7);
  axis = normalize(vec3(sin(uTime*0.4 + i), 1.6, cos(uTime*0.3 + i*2.0)));
  return around(3.4 + 0.8*sin(i*2.3), a, y);
}
// round 10: the crystals circle the blade, so they are drawn in true 3D (only orbits are)
vec3 base_orbit(vec3 ro, vec3 rd, float px){
  float t = uTime, top = kc_top(), s, ta;
  float cl = clamp(uClash, 0.0, 1.0);
  vec3 col = vec3(0.0);
  if (top < 0.5) return col;
  for (int i = 0; i < 9; i++){
    float fi = float(i);
    vec3 ax;
    vec3 c = kc_pos(fi, ax);
    if (c.y > top) continue;
    vec3 kcol = kc_col(fi);
    float fl = pow(0.5 + 0.5*sin(t*1.3 + fi*2.7), 10.0) + cl;
    // its glow
    float d = rayPt(ro, rd, c, ta);
    col += kcol*(halo(d, 0.6)*0.35 + halo(d, 1.4)*0.08)*(1.0 + 2.0*fl);
    // the beam to the blade, pulsing
    vec3 e = vec3(0.0, c.y, 0.0);
    float db = raySeg(ro, rd, c, e, s);
    float pul = 0.3 + 0.7*pow(0.5 + 0.5*sin(s*10.0 + t*6.0 + fi), 4.0);
    col += kcol*(halo(db, max(0.025, px))*0.9 + halo(db, 0.15)*0.15)*pul*(0.6 + fl);
    // a bead of its colour racing into the blade when it flares
    float bp = fract(t*0.7 + fi*0.31);
    float dbd = rayPt(ro, rd, mix(c, e, bp), ta);
    col += mix(kcol, vec3(1.0), 0.4)*halo(dbd, max(0.08, px))*2.0*step(0.6, h11(floor(t*0.7 + fi*0.31) + fi));
  }
  return col*uFx;
}
// the crystals themselves: elongated shards facing the camera, faceted, lit in their colour
vec4 base_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 o = vec4(0.0);
  if (uFx < 0.01) return o;
  float top = kc_top(), bestT = 1e5;
  // the blade hides the crystals behind it (round 12: they showed through it)
  float sAx, tAx;
  float dAx = raySegRaw(ro, rd, vec3(0.0, 0.0, 0.0), vec3(0.0, max(top, 0.01), 0.0), sAx);
  float dAx2 = rayPtRaw(ro, rd, vec3(0.0, sAx*top, 0.0), tAx);
  float hide = 1.0 - smoothstep(0.45, 1.1, dAx2);
  for (int i = 0; i < 9; i++){
    float fi = float(i);
    vec3 ax;
    vec3 c = kc_pos(fi, ax);
    if (c.y > top) continue;
    float ln = 1.8 + 0.8*h11(fi*3.3);
    float s, ta;
    float d = raySeg(ro, rd, c - ax*ln, c + ax*ln, s);
    float w = 0.62*(1.0 - pow(abs(s*2.0 - 1.0), 1.5)) + 0.01;
    float cv = fillD(d - w, px);
    if (cv <= 0.0) continue;
    rayPt(ro, rd, c, ta);
    if (ta > bestT) continue;
    if (ta > tAx) cv *= 1.0 - hide;
    if (cv <= 0.0) continue;
    bestT = ta;
    vec3 kcol = kc_col(fi);
    float facet = step(0.0, sin(s*6.0 + fi))*0.25 + 0.75;
    float edge = smoothstep(w*0.5, w, d);
    float fl = pow(0.5 + 0.5*sin(uTime*1.3 + fi*2.7), 10.0) + clamp(uClash, 0.0, 1.0);
    vec3 rgb = kcol*(0.5 + 0.5*facet)*(0.8 + 1.2*fl) + vec3(1.0)*edge*0.6;
    o = vec4(mix(o.rgb, rgb, cv*0.9), max(o.a, cv*0.9));
  }
  return o;
}
// the blade: Sol's glow and core from the file with facets glinting in it
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float f = abs(fract(b.y*0.4 + b.x*0.8) - 0.5);
  float gl = pow(0.5 + 0.5*sin(uTime*2.0 + floor(b.y*0.4 + b.x*0.8)*1.7), 12.0);
  return g*(0.9 + 0.3*halo(f, 0.05)*gl + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }

// round 12 (Creyon: "needs something more"): the crystals' song, motes of every crystal's
// colour rising round the blade (round 15: without the streams it shared with the Aurodium)
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  for (int k = 0; k < 3; k++){
    float fk = float(k);
    fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 10.0, 0.05, 3.1 + fk*2.7, kc_col(fk*3.0))*1.0;
  }
  // round 13 (Creyon): a chroma for the blade only, the crystals keep their own colours (uKeep)
  uKeep += fx*uFx*part(1.0);
  return vec3(0.0);
}
// round 11: sig_orbit gated by part 0
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  uKeep += base_orbit(ro, rd, px);
  return vec3(0.0);
}
// round 11: sig_orbit_solid gated by part 0
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec4(0.0); }
  vec4 o = base_orbit_solid(ro, rd, px);
  uOwn = max(uOwn, o.a);
  return o;
}
