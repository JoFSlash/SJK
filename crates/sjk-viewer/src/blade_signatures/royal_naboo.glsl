// Royal Naboo: the grace of Naboo's royal court as a blade. A white-gold blade traced with golden
// filigree that slowly flows up it, a thread of royal crimson at its heart; a queen's veil of golden
// silk flutters round its upper half in 3D (round 17), soft petals of gold light drift down past it
// and a warm royal glow surrounds it. A clash sends a gold ring.
// @part Golden filigree
// @part Petals of light
// @part Royal glow
// @room 8
// @palette {"corona.rim_cool":[0.6,0.45,0.15],"corona.rim_hot":[1.0,0.85,0.5],"corona.inner":[1.0,0.97,0.9],"core.white":[1,1,0.96],"core.fringe_cool":[1.0,0.8,0.45],"core.fringe_hot":[1.0,0.93,0.75],"light.color":[1.0,0.85,0.55],"trail":[1.0,0.85,0.55]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  // round 17: two interlaced scrolls of gold filigree, brighter, and a thread of royal crimson
  // glowing at its heart
  float f1 = halo(abs(sin(b.y*0.9 - uTime*0.6 + 3.0*sin(b.x*6.0))), 0.12);
  float f2 = halo(abs(sin(b.y*0.9 + 1.57 - uTime*0.6 - 3.0*sin(b.x*6.0))), 0.1);
  float fil = max(f1, f2)*part(0.0);
  float heart = exp(-pow(b.x/0.06, 2.0))*(0.6 + 0.4*sin(b.y*0.3 - uTime*0.8));
  return recol(g, vec3(1.0, 0.95, 0.82)) + lvl(g)*(vec3(1.0, 0.72, 0.25)*fil*1.1 + vec3(0.9, 0.12, 0.15)*heart*0.5);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 gold = vec3(1.0, 0.78, 0.35), rose = vec3(1.0, 0.7, 0.55);
  fx += fxFlakes(ro, rd, px, y0, y1, 3.0, 8.0, 2.2, 0.4, 0.5, 0.8, 0.8, 4.7, rose)*part(1.0);
  fx += fxAura(ro, rd, y0, y1, 2.4, gold)*0.4*part(2.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 22.0, 0.03, 6.8, vec3(1.0, 0.9, 0.6))*part(2.0);
  return fx*uFx;
}
// round 17 (Creyon: it looked like something was missing, and no small figures): a queen's veil of
// golden silk, a broad translucent sheet fluttering round the blade's upper half with a bright
// gold hem, rippling as it turns; it circles the blade, so it is drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, L*0.3, L*0.98) > 4.5) return fx;
  vec3 gold = vec3(1.0, 0.8, 0.4), silk = vec3(1.0, 0.9, 0.7);
  float hemB = 1e3, hemT = 1e3, mid = 1e3, bm = 0.0;
  vec3 pb = vec3(0.0), pt = vec3(0.0), pm = vec3(0.0);
  for (int i = 0; i <= 18; i++){
    float u = float(i)/18.0;
    float a = u*0.75 + t*0.05;
    float R = 1.9 + 0.35*sin(u*9.0 - t*1.6);
    float y = L*0.45 + u*L*0.35 + 0.8*sin(u*6.0 - t*1.2);
    vec3 b = around(R, a, y), tp = around(R + 0.15, a, y + 2.2), m = around(R + 0.08, a, y + 1.1);
    if (i > 0){
      hemB = min(hemB, raySeg(ro, rd, pb, b, s));
      hemT = min(hemT, raySeg(ro, rd, pt, tp, s));
      float dm = raySeg(ro, rd, pm, m, s);
      if (dm < mid){ mid = dm; bm = (float(i) - 1.0 + s)/18.0; }
    }
    pb = b; pt = tp; pm = m;
  }
  float ends = smoothstep(0.0, 0.12, bm)*(1.0 - smoothstep(0.85, 1.0, bm));
  float sheet = halo(mid, 1.0)*0.28;
  float sheen = 0.6 + 0.4*sin(bm*20.0 - t*2.0);
  fx += silk*sheet*sheen*ends + gold*(halo(hemB, max(0.04, px))*1.2 + halo(hemB, 0.18)*0.3)*ends
        + gold*halo(hemT, max(0.03, px))*0.5*ends;
  return fx*uFx;
}
