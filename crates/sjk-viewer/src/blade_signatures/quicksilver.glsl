// Liquid Beskar: Mandalorian steel kept molten, as a blade (Creyon's round 9: its droplets and
// sparks in 3D). A thin blade of liquid mirror: a chrome surface reflecting a cool sky and warm
// floor, ripples racing up it, beads of beskar rolling up to the tip and dripping off. Round it in
// 3D, droplets of liquid beskar orbit and drip, silver glints fly off it and the air shimmers. A
// swing flings a spray of silver droplets off the blade that arc away in 3D; a clash throws a ring
// of droplets out with a shock ring; ignition pours the metal up from the hilt. Round 15 (Creyon:
// it needed more; round 16: no forge fire): rings of liquid beskar ripple up round the mirror
// (round 19, Creyon: beskar has no tendrils, in the films or the comics; rings instead).
// @full 1
// @part Silver glints
// @part Flung droplets
// @part Orbiting droplets
// @part Beskar ripples
// @room 10
// @solid 1

float qs_w(float y){ float L = uLength; return 0.5*smoothstep(-0.3, 0.5, y)*(1.0 - smoothstep(L - 0.9, L + 0.05, y)) + 0.05; }
// fake chrome: what the curved mirror reflects at a slope n (-1..1 across) and height y
vec3 qs_env(float n, float y, float t){
  float h = n*0.9 + 0.12*sin(y*0.35 - t*1.3);
  vec3 sky = mix(vec3(0.55, 0.65, 0.85), vec3(0.95, 0.97, 1.0), smoothstep(-0.1, 0.6, h));
  vec3 floor_ = mix(vec3(0.12, 0.1, 0.12), vec3(0.4, 0.33, 0.3), smoothstep(-1.0, -0.3, h));
  vec3 c = mix(floor_, sky, smoothstep(-0.25, 0.05, h));
  c += vec3(1.0)*exp(-pow((h - 0.45)/0.06, 2.0))*1.2;     // the horizon highlight
  return c;
}
// a mercury drop: sphere shading with a highlight, returns (rgb, coverage)
vec4 qs_drop(vec2 q, float rad, float px){
  float d = length(q);
  float cov = fillD(d - rad, px);
  if (cov <= 0.0) return vec4(0.0);
  vec2 n = q/max(rad, 1e-3);
  vec3 c = qs_env(n.x, n.y*3.0, 0.0)*(0.65 + 0.35*sqrt(max(1.0 - dot(n, n), 0.0)));
  c += vec3(1.0)*exp(-dot(n - vec2(-0.35, 0.4), n - vec2(-0.35, 0.4))*18.0)*1.3;
  return vec4(c, cov);
}
// flung droplet k (spray off the blade): position and radius, alive amount
vec4 qs_fling(float fk, float t){
  float L = uLength;
  float rate = 0.4 + 0.25*h11(fk*2.3);
  float ph = t*rate + h11(fk*8.1);
  float cy = floor(ph), f = fract(ph);
  float live = step(h11(cy*2.9 + fk), 0.45 + 0.55*uSwing + 0.5*uClash);
  float s = h11(cy*4.1 + fk) < 0.5 ? -1.0 : 1.0;
  float y0 = (0.15 + 0.8*h11(cy*1.7 + fk*3.0))*L;
  vec2 v = vec2(s*(4.0 + 4.0*h11(cy + fk*5.0)), 2.5*(h11(cy*6.0 + fk) - 0.2));
  vec2 p = vec2(s*0.4, y0) + v*f - vec2(0.0, 3.5*f*f);
  return vec4(p, (0.3 + 0.25*h11(fk*4.4))*(1.0 - 0.4*f), live*(1.0 - smoothstep(0.8, 1.0, f)));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float L = uLength;
  px = min(px, 0.06);
  float on = smoothstep(-0.6, 0.3, b.y);
  float edge = 1.0 - smoothstep(6.0, 8.5, abs(b.x));
  float yc = clamp(b.y, 0.0, L - 0.2);
  float d = length(vec2(b.x, b.y - yc));
  // a cool silver sheen round the mirror, ripples brightening it as they pass
  float rip = 0.5 + 0.5*sin(b.y*1.6 - uTime*9.0);
  vec3 light = vec3(0.75, 0.82, 0.95)*halo(d, 0.9)*(0.35 + 0.25*rip)*on + vec3(0.6, 0.7, 0.9)*halo(d, 2.0)*0.08*on;
  light *= edge*(1.0 - smoothstep(L + 2.0, L + 6.0, b.y))*smoothstep(-3.0, -0.5, b.y);
  return (1.0 - exp(-light*1.1))*1.05*clamp(uIgn*1.5, 0.0, 1.0);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return vec3(0.0); }
vec4 sig_solid(vec4 o, vec2 b, float px){
  px = min(px, 0.06);
  float L = uLength;
  float ign = smoothstep(0.0, 1.5, uAge);
  float fillTo = mix(-0.5, L + 0.1, ign);                       // ignition: the metal pours up
  // ripples bulge the surface: the width wobbles where a ripple passes
  float rip = sin(b.y*1.6 - uTime*9.0)*0.5 + 0.5;
  float w = qs_w(b.y)*(1.0 + 0.12*rip);
  float cov = fillD(abs(b.x) - w, px)*step(-0.3, b.y)*fillD(b.y - min(L + 0.05, fillTo), px);
  vec4 acc = o;
  if (cov > 0.0){
    float n = b.x/max(w, 0.01);
    float slope = n + 0.25*cos(b.y*1.6 - uTime*9.0);           // ripples tilt what it reflects
    vec3 c = qs_env(slope, b.y, uTime);
    c *= 0.6 + 0.4*sqrt(max(1.0 - n*n, 0.0));
    c += vec3(1.0)*exp(-pow((b.y - mod(uTime*14.0, L + 10.0) + 4.0)/0.6, 2.0))*0.5;   // a sliding glint
    acc = vec4(mix(acc.rgb, c, cov), max(acc.a, cov));
  }
  // beads rolling up the blade, swelling at the tip and dripping off
  for (int k = 0; k < 5; k++){
    float fk = float(k);
    float per = 3.0 + fk*0.7;
    float tt = mod(uTime + fk*1.37, per)/per;
    float y = tt*(L + 1.0) - 0.5;
    float side = mod(fk, 2.0) < 0.5 ? 1.0 : -1.0;
    float rad = 0.38 + 0.12*h11(fk);
    vec2 p = vec2(side*qs_w(y)*0.9, y);
    // past the tip: it drips off and falls away in an arc
    float past = max(y - L, 0.0);
    p += vec2(side*past*1.4, past*0.3 - past*past*0.9);
    vec4 dr = qs_drop(b - p, rad*(1.0 + 0.4*smoothstep(L - 3.0, L, y)), px);
    dr.a *= (1.0 - smoothstep(L + 0.5, L + 1.0, y))*ign;
    acc = vec4(mix(acc.rgb, dr.rgb, dr.a), max(acc.a, dr.a));
  }
  acc.a *= clamp(uIgn*1.5, 0.0, 1.0);
  return acc;
}

// ---- round 9: the droplets in 3D, mirror spheres ----
// droplet k flung off the blade: centre (xyz) and radius (w); alive when w > 0
vec4 qs_fling3(float fk){
  float L = uLength;
  float rate = 0.4 + 0.25*h11(fk*2.3);
  float ph = uTime*rate + h11(fk*8.1);
  float cy = floor(ph), f = fract(ph);
  float live = step(h11(cy*2.9 + fk), 0.25 + 0.75*uSwing + 0.5*uClash)*(1.0 - smoothstep(0.8, 1.0, f));
  float a = h11(cy*4.1 + fk);
  float y0 = (0.2 + 0.75*h11(cy*1.7 + fk*3.0))*L;
  vec3 dir = vec3(sin(6.2832*a), 0.5*(h11(cy*6.0 + fk) - 0.2), cos(6.2832*a));
  vec3 p = vec3(0.0, y0, 0.0) + dir*0.5 + dir*(4.0 + 4.0*h11(cy + fk*5.0))*f - vec3(0.0, 3.5*f*f, 0.0);
  return vec4(p, (0.3 + 0.25*h11(fk*4.4))*(1.0 - 0.4*f)*live);
}
// droplet k orbiting slowly round the blade, bobbing like molten metal held by the heat
vec4 qs_orbit3(float fk){
  float L = uLength;
  float a = h11(fk*1.9) + uTime*(0.06 + 0.05*h11(fk*3.1));
  float y = (0.22 + 0.72*fract(fk*0.413 + 0.11))*L + 0.7*sin(uTime*0.8 + fk*2.0);
  return vec4(around(2.0 + 2.5*h11(fk*5.7), a, y), 0.22 + 0.2*h11(fk*7.3));
}
// a mirror sphere: (rgb, coverage) where the ray hits it, t its distance
vec4 qs_sphere(vec3 ro, vec3 rd, vec4 c, float px, out float th){
  float d = rayPt(ro, rd, c.xyz, th);
  if (c.w <= 0.0 || d > c.w + px) return vec4(0.0);
  float cov = clamp((c.w - d)/max(px, 1e-3) + 0.5, 0.0, 1.0);
  float back = sqrt(max(c.w*c.w - d*d, 0.0));
  vec3 n = normalize(ro + rd*(th - back) - c.xyz);
  vec3 col = qs_env(clamp(n.y*0.9 + n.x*0.3, -1.0, 1.0), c.y, uTime)*(0.55 + 0.45*max(dot(n, -rd), 0.0));
  col += vec3(1.0)*pow(max(dot(reflect(rd, n), normalize(vec3(-0.3, 0.8, 0.5))), 0.0), 40.0)*1.5;
  return vec4(col, cov);
}
vec4 base_vol_solid(vec3 ro, vec3 rd, float px){
  vec4 acc = vec4(0.0);
  if (uFx < 0.01 || fxNear(ro, rd, uLength*0.1, uLength) > 9.0) return acc;
  float best = 1e5, th;
  for (int k = 0; k < 12; k++){
    vec4 c = qs_fling3(float(k));
    vec4 dr = qs_sphere(ro, rd, c, px, th);
    if (dr.a > 0.0 && th < best){ best = th; acc = dr; }
  }
  // a clash: a ring of droplets blown out
  float cl = clamp(uClash, 0.0, 1.0);
  if (cl > 0.01){
    float a = 1.0 - cl;
    for (int k = 0; k < 10; k++){
      float fk = float(k);
      vec3 p = vec3(0.0, uLength*0.55, 0.0) + around(0.8 + 6.0*(1.0 - exp(-3.0*a)), fk/10.0, -2.0*a*a);
      vec4 dr = qs_sphere(ro, rd, vec4(p, 0.3*(1.0 - 0.5*a)), px, th);
      if (dr.a > 0.0 && th < best){ best = th; acc = dr; }
    }
  }
  acc.a *= clamp(uIgn*1.5, 0.0, 1.0);
  return acc;
}
vec3 base_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 silver = vec3(0.8, 0.88, 1.0);
  vec3 col = vec3(0.0);
  // round 17 (Creyon: nothing red, it is pure beskar, only liquid): silver glints, not forge sparks
  col += fxSparks(ro, rd, px, y0, y1, 16.0 + 14.0*sw, 6.0, 1.0, 3.7, silver);
  col += fxAura(ro, rd, y0, y1, 2.6, silver)*0.6;
  col += fxShock(ro, rd, px, L*0.55, 1.0 - cl, 9.0, silver)*1.5;
  return col*uFx;
}
// round 10: the droplets circling the blade, in true 3D (only orbits are; the rest is flat)
vec4 base_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 acc = vec4(0.0);
  if (uFx < 0.01 || fxNear(ro, rd, uLength*0.1, uLength) > 9.0) return acc;
  float best = 1e5, th;
  for (int k = 0; k < 10; k++){
    vec4 dr = qs_sphere(ro, rd, qs_orbit3(float(k)), px, th);
    if (dr.a > 0.0 && th < best){ best = th; acc = dr; }
  }
  acc.a *= clamp(uIgn*1.5, 0.0, 1.0);
  return acc;
}
// round 11: sig_vol gated by part 0
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  return base_vol(ro, rd, px);
}
// round 11: sig_vol_solid gated by part 1
vec4 sig_vol_solid(vec3 ro, vec3 rd, float px){
  vec4 acc = vec4(0.0);
  if (part(3.0) > 0.5 && uFx > 0.01) { acc = qs_ripples(ro, rd, px); }
  if (part(1.0) < 0.5) { return acc; }
  vec4 d = base_vol_solid(ro, rd, px);
  return vec4(mix(acc.rgb, d.rgb, d.a), max(acc.a, d.a));
}
// round 11: sig_orbit_solid gated by part 2
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec4(0.0); }
  return base_orbit_solid(ro, rd, px);
}
// round 19 (Creyon: beskar has no tendrils): rings of liquid beskar rippling up round the
// blade, as a wave runs up a column of molten metal: each a thin mirror band swelling out from
// the surface, travelling up, and sinking back into it before the tip; in three dimensions
vec4 qs_ripples(vec3 ro, vec3 rd, float px){
  vec4 acc = vec4(0.0);
  float L = uLength, t = uTime;
  if (fxNear(ro, rd, 0.0, L) > 3.0) return acc;
  float best = 1e5;
  for (int j = 0; j < 3; j++){
    float fj = float(j);
    float q = fract(t*0.16 + fj/3.0);
    float y = mix(L*0.08, L*0.9, q);
    float swell = sin(3.1416*q);
    float R = 0.55 + 1.1*swell;
    float w = 0.09 + 0.1*swell;
    float d = ringD(ro, rd, y, R);
    float cov = fillD(d - w, px)*smoothstep(0.02, 0.15, swell);
    if (cov <= 0.0) continue;
    // where along the ray it is, so the nearer ring covers the farther
    vec3 h = rayAt(ro, rd, y);
    float th = h.z > 0.0 ? h.z : 1e4;
    // round 22 (Creyon: the rings were seen through the blade): the half behind the blade is
    // hidden by it
    cov *= h.z > 0.0 ? bladeSeen(ro, rd, vec3(h.x, y, h.y)) : 1.0;
    if (cov <= 0.0) continue;
    float n = clamp(d/max(w, 1e-3), 0.0, 1.0);
    vec3 c = qs_env(0.6 - n*1.2, y, t)*(0.7 + 0.3*sqrt(max(1.0 - n*n, 0.0)));
    c += vec3(1.0)*exp(-n*n*6.0)*0.35;
    if (acc.a <= 0.0 || th < best){ best = th; acc = vec4(mix(acc.rgb, c, cov), max(acc.a, cov)); }
  }
  return acc;
}
