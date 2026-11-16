// Krayt Pearl: the pearl of a great krayt dragon, as a blade (round 16 rework, Creyon: it did not
// look like a legendary). The blade is mother-of-pearl: soft swirls of nacre whose colours shift
// pink, gold and blue as they turn, under a smooth sheen; round it, krayt pearls circle slowly in
// 3D, each a lustrous ball catching the light, grains of desert sand stream past on the wind and a
// veil of sand dust curls off it. A swing brightens the pearl; a clash a pale ring.
// @part Pearls
// @part Desert sand
// @part Sand veil
// @room 8
// @solid 1
// @palette {"corona.rim_cool":[0.55,0.5,0.4],"corona.rim_hot":[1.0,0.92,0.8],"corona.inner":[1.0,0.98,0.94],"core.white":[1,1,1],"core.fringe_cool":[0.95,0.85,0.7],"core.fringe_hot":[1.0,0.96,0.9],"light.color":[1.0,0.92,0.8],"trail":[1.0,0.92,0.8]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  // nacre: swirling bands whose thin-film colours shift as they turn
  float sw = fbm3(vec2(b.x*2.0 + 0.4*sin(b.y*0.2 + uTime*0.3), b.y*0.18 - uTime*0.12));
  float bandv = 0.5 + 0.5*sin((sw*5.0 + b.x*2.0)*3.1416 + uTime*0.4);
  vec3 film = hsv(fract(0.92 + 0.25*sw + 0.1*bandv), 0.35, 1.0);
  float sheen = halo(abs(b.x) - 0.12, 0.06)*0.6;
  return lvl(g)*(mix(vec3(1.0, 0.96, 0.92), film, 0.6) + vec3(1.0)*sheen)*(0.9 + 0.3*bandv + 0.4*clamp(uSwing, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 sand = vec3(1.0, 0.85, 0.6);
  fx += fxFall(ro, rd, px, y0, y1, 3.5, 16.0, 6.0, 0.4, 1.4, 6.2, sand)*0.8*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.4, 1.0, 6.6, sand*0.7)*part(2.0);
  fx += fxAura(ro, rd, y0, y1, 2.4, vec3(1.0, 0.85, 0.9))*0.3*part(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 4.0, 12.0, 7.0, 0.5, 1.6, 2.6, vec3(1.0, 0.85, 0.6))*0.6*part(1.0);
  return fx*uFx;
}
// round 16: krayt pearls circling the blade slowly, lustrous balls catching the light; they circle
// it, so they are drawn in true 3D round it (Sol's way)
vec4 kp_pearl(float fk){
  float L = uLength;
  return vec4(around(2.6 + 0.6*h11(fk*3.1), fk/5.0 + uTime*(0.05 + 0.02*h11(fk)), L*(0.3 + 0.13*fk) + 0.5*sin(uTime*0.6 + fk*2.0)), 0.5 + 0.12*h11(fk*5.7));
}
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 acc = vec4(0.0);
  if (part(0.0) < 0.5 || uFx < 0.01) return acc;
  float best = 1e5, th;
  for (int k = 0; k < 5; k++){
    vec4 c = kp_pearl(float(k));
    float d = rayPt(ro, rd, c.xyz, th);
    if (d > c.w + px || th > best) continue;
    float cov = fillD(d - c.w, px);
    float back = sqrt(max(c.w*c.w - d*d, 0.0));
    vec3 n = normalize(ro + rd*(th - back) - c.xyz);
    float fres = pow(1.0 - max(dot(n, -rd), 0.0), 2.0);
    vec3 film = hsv(fract(0.9 + 0.3*n.y + 0.2*n.x + uTime*0.03), 0.35, 1.0);
    vec3 col = mix(vec3(1.0, 0.97, 0.93), film, 0.4 + 0.5*fres)*(0.95 + 0.45*max(n.y, 0.0));
    col += vec3(1.0)*pow(max(dot(reflect(rd, n), normalize(vec3(-0.3, 0.8, 0.5))), 0.0), 30.0)*1.2;
    // round 17: hidden behind the blade
    acc = vec4(col, cov*bladeSeen(ro, rd, c.xyz)); best = th;
  }
  uOwn = max(uOwn, acc.a);
  return acc;
}
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  vec3 acc = vec3(0.0);
  float th;
  for (int k = 0; k < 5; k++){
    vec4 c = kp_pearl(float(k));
    float d = rayPt(ro, rd, c.xyz, th);
    acc += vec3(1.0, 0.9, 0.95)*halo(d - c.w, 0.35)*0.3;
  }
  return acc*uFx;
}
