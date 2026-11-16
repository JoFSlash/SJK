// Assassin Droid: an assassin droid's weapon as a blade. Its light is cut into hard plated bands
// that shift like machinery; a red scanner ring sweeps up and down it while red targeting lasers
// flick out of it to lock on and blink off, two armoured gear rings of plates turn round it in
// opposite directions, sparks spit off its joints and a thin grey smoke rises off it (round 15,
// Creyon: it looked cheaper than lower tiers). A clash flares the bands.
// @part Scanner
// @part Sparks
// @part Droid smoke
// @part Targeting lasers
// @part Gear rings
// @room 8
// @palette {"corona.rim_cool":[0.5,0.05,0.02],"corona.rim_hot":[1.0,0.3,0.1],"corona.inner":[1.0,0.85,0.7],"core.white":[1,0.95,0.9],"core.fringe_cool":[1.0,0.3,0.1],"core.fringe_hot":[1.0,0.75,0.55],"light.color":[1.0,0.35,0.12],"trail":[1.0,0.35,0.12]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float seg = fract(b.y/2.5 + floor(uTime*0.8 + h11(floor(b.y/2.5)))*0.0);
  float joint = halo((0.5 - abs(seg - 0.5))*2.5, 0.06);
  float scan = halo(b.y - (0.5 + 0.5*sin(uTime*0.9))*uLength, 0.5)*part(0.0);
  return g*(0.85 + 0.5*scan + 0.6*clamp(uClash, 0.0, 1.0)) + lvl(g)*vec3(1.0, 0.85, 0.7)*joint*0.6;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(1.0, 0.75, 0.5), red = vec3(1.0, 0.15, 0.1);
  float ys = mix(y0, y1, 0.5 + 0.5*sin(t*0.9));
  float d = ringD(ro, rd, ys, 2.2);
  fx += (vec3(1.0, 0.6, 0.55)*halo(d, max(0.04, px))*1.1 + red*halo(d, 0.4)*0.8)*part(0.0);
  fx += fxSparks(ro, rd, px, y0, y1, 12.0 + 10.0*sw, 3.5, 1.4, 2.7, c)*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.6, 2.5, 4.2, vec3(0.45, 0.45, 0.5))*0.8*part(2.0);
  // targeting lasers: thin red beams flicking out of the blade at a target, a lock-on dot at the
  // end blinking twice, then gone
  float s, ta;
  if (fxNear(ro, rd, y0, y1) < 8.0){
    for (int i = 0; i < 4; i++){
      float fi = float(i);
      float cyc = t*(0.55 + 0.2*h11(fi*3.3)) + h11(fi*7.1);
      float k = floor(cyc), q = fract(cyc);
      if (q > 0.55) continue;
      float a = h11(k*2.9 + fi);
      float y = mix(y0, y1, h11(k*5.3 + fi*1.7));
      vec3 o = around(0.35, a, y);
      vec3 e = o + normalize(vec3(sin(6.2832*a), (h11(k*1.1 + fi) - 0.5)*0.6, cos(6.2832*a)))*(4.5 + 2.0*h11(k + fi));
      float grow = smoothstep(0.0, 0.08, q);
      float d = raySeg(ro, rd, o, mix(o, e, grow), s);
      float env = 1.0 - smoothstep(0.4, 0.55, q);
      fx += (vec3(1.0, 0.7, 0.7)*halo(d, max(0.02, px))*1.3 + red*halo(d, 0.1)*0.7)*env*part(3.0);
      float lock = step(0.5, fract(q*7.0))*step(0.12, q);
      float dl = rayPt(ro, rd, e, ta);
      fx += (vec3(1.0, 0.8, 0.8)*halo(dl, max(0.06, px))*2.0 + red*halo(dl, 0.4)*0.6
             + red*halo(abs(dl - 0.45), 0.04)*1.2)*env*lock*grow*part(3.0);
    }
  }
  return fx*uFx;
}
// round 15: two armoured rings of plates turning round the blade in opposite directions, a red
// light blinking on each; they circle it, so they are drawn in 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(4.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, L*0.2, L*0.9) > 4.0) return fx;
  for (int k = 0; k < 2; k++){
    float fk = float(k);
    float y = L*(0.32 + 0.4*fk);
    float R = 2.4;
    float spin = t*(k == 0 ? 0.18 : -0.24);
    for (int i = 0; i < 8; i++){
      float fi = float(i);
      float a0 = fi/8.0 + spin, a1 = a0 + 0.085;
      float d = raySeg(ro, rd, around(R, a0, y), around(R, a1, y), s);
      float d2 = raySeg(ro, rd, around(R, a0, y + 0.6), around(R, a1, y + 0.6), s);
      fx += vec3(0.95, 0.75, 0.6)*(halo(min(d, d2), max(0.06, px))*1.3 + halo(min(d, d2), 0.25)*0.3);
    }
    float ta;
    float dl = rayPt(ro, rd, around(R, spin + 0.04, y + 0.3), ta);
    float blink = step(0.5, fract(t*1.5 + fk*0.5));
    fx += vec3(1.0, 0.15, 0.1)*(halo(dl, max(0.07, px))*2.0 + halo(dl, 0.35)*0.5)*blink;
  }
  return fx*uFx;
}
