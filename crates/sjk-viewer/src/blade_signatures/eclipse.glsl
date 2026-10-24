// Eclipse: a thin blade (pearl-white, or its wearer's colour: a chroma) with a total eclipse
// travelling along it. Where the moon passes, a dark ring crosses the blade and its light goes
// out in a band. The band drifts slowly up and down the blade, with diamond-ring flashes and
// Baily's beads sparking on its edge, while small pearly prominences loop up off the blade and
// fade, and the stars of totality come out round the band. Ignites fast; a clash flares the
// band with a diamond-ring burst.
// @part Totality
// @part Eclipsed suns
// @arcs Prominences
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.75,0.62,0.40],"corona.rim_hot":[1.0,0.90,0.70],"corona.inner":[1.0,0.97,0.90],"core.white":[1.0,0.98,0.92],"core.fringe_cool":[0.95,0.75,0.45],"core.fringe_hot":[1.0,0.92,0.75],"light.color":[1.0,0.86,0.62],"trail":[1.0,0.84,0.58]}

float ec_top(){ return uLength*clamp(uAge/0.4, 0.0, 1.0); }
// the band's height: drifting up and down the blade, rising out of the hilt at ignition
float ec_y(){
  float L = uLength;
  // kept in the blade's upper half, clear of its wearer's head and body (Creyon's round 9)
  float y = L*(0.7 + 0.16*sin(uTime*0.3 + uSeed*6.0));
  return mix(L*0.5, y, smoothstep(0.2, 1.2, uAge));
}
float ec_hw(){ return 1.4; }
float ec_ringR(){ return 0.55; }
// how dark the blade is at height y (1 in the band)
float ec_shade(float y){
  float yb = ec_y(), hw = ec_hw();
  return (1.0 - smoothstep(hw*0.45, hw*0.8, abs(y - yb)))*smoothstep(0.0, 0.5, uAge);
}

// the eclipsed suns: three dark moons, each in front of its own small sun, orbiting the blade
// in 3D like the Void's black holes: centre and radius of orb i
vec3 ec_orbAt(float i, float tt, out float R){
  float L = uLength;
  R = 0.9 + 0.25*i;
  float a = i/3.0 + tt*(0.12 + 0.05*i)*(i == 1.0 ? -1.0 : 1.0);
  float y = L*(0.5 + 0.17*i) + 1.0*sin(tt*0.4 + i*2.0);
  return around(3.4 + 0.4*i, a, y);
}
vec3 ec_orb(float i, out float R){ return ec_orbAt(i, uTime, R); }
// a point's place in the plane facing the camera round centre c: (across, up) and distance
vec3 ec_face(vec3 ro, vec3 rd, vec3 c){
  float t;
  float d = rayPt(ro, rd, c, t);
  vec3 off = ro + rd*t - c;
  vec3 ux = normalize(cross(rd, vec3(0.0, 1.0, 0.0)) + vec3(1e-4, 0.0, 0.0));
  vec3 uy = cross(ux, rd);
  return vec3(dot(off, ux), dot(off, uy), d);
}
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, top = ec_top(), s, tq;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float flare = max(cl, exp(-uAge*1.6)*step(0.0, uAge)*0.35);
  vec3 pearl = vec3(1.0, 0.9, 0.72), white = vec3(1.0, 0.98, 0.94);
  float yb = ec_y(), hw = ec_hw(), R0 = ec_ringR();
  // The blade is Sol's glow and core from the file, dark in the moon's band (sig_glow,
  // sig_core, sig_solid); here only the effects, each a piece placed round it in 3D.
  vec3 light = vec3(0.0);
  vec3 fx = vec3(0.0);
  float vis = smoothstep(0.1, 0.6, uAge);
  // (round 14, Creyon: no corona of streamers round the band; instead) the stars of totality:
  // as the band darkens the sky, small stars come out round it and twinkle, the nearest the band
  // the brightest
  for (int i = 0; i < 22; i++){
    float fi = float(i);
    vec3 sp = around(1.2 + 4.2*h11(fi*2.9), h11(fi*6.3), yb + (h11(fi*4.1) - 0.5)*7.0);
    float ds = rayPt(ro, rd, sp, tq);
    float tw = 0.55 + 0.45*sin(uTime*(1.5 + 2.0*h11(fi*8.3)) + fi*3.7);
    float near = 1.0 - smoothstep(2.0, 4.5, abs(sp.y - yb));
    fx += mix(white, vec3(0.75, 0.85, 1.0), h11(fi*1.1))*(halo(ds, max(0.04, px))*1.6 + halo(ds, 0.18)*0.25)*tw*near*vis;
  }
  // ---- solar wind: pearly motes streaming out from the band and the suns
  for (int i = 0; i < 18; i++){
    float fi = float(i);
    float f = fract(uTime*0.35 + h11(fi*3.3));
    float a = h11(fi*7.1 + floor(uTime*0.35 + h11(fi*3.3)));
    vec3 p = around(0.6 + 5.0*f, a + 0.1*f, yb + (h11(fi*5.9) - 0.5)*2.5*f);
    float dm = rayPt(ro, rd, p, tq);
    fx += white*halo(dm, max(0.05, px))*1.2*sin(3.1416*f)*vis;
  }
  // ---- the chromosphere: thin bright rims along both edges of the band, round the blade
  for (int k = 0; k < 2; k++){
    float ey = yb + (float(k)*2.0 - 1.0)*hw*0.62;
    vec3 hp = rayAt(ro, rd, ey);
    if (hp.z > 0.0){
      // round 23: hidden behind the blade softly (see hideD in lab.glsl)
      float rr = length(hp.xy), seen = 1.0 - behindBlade(ro, rd, hp.z);
      float rim = halo(abs(rr - 0.42), max(0.05, px))*1.1;
      fx += white*rim*vis*(1.0 - smoothstep(0.6, 1.2, rr))*seen;
    }
  }
  // ---- diamond ring and Baily's beads: flashes on the band's edge as the moon slides
  float cyc = uTime*0.45, f = fract(cyc), id = floor(cyc);
  float fl = exp(-f*5.0)*smoothstep(0.0, 0.03, f) + flare*0.8;
  float upper = h11(id*3.7) < 0.5 ? 1.0 : -1.0;
  float a0 = h11(id*1.9);
  vec3 dp = around(0.42, a0, yb + upper*hw*0.62);
  float dd = rayPt(ro, rd, dp, tq);
  fx += white*(halo(dd, 0.12)*2.5 + halo(dd, 0.6)*0.6 + halo(dd, 2.0)*0.15)*fl*vis;
  // a soft horizontal flash streak through the diamond, round the band
  vec3 hp2 = rayAt(ro, rd, dp.y);
  if (hp2.z > 0.0){
    float rr = length(hp2.xy), seen = 1.0 - behindBlade(ro, rd, hp2.z);
    fx += white*halo(abs(rr - 0.42), 0.25)*exp(-rr/2.5)*fl*0.6*vis*seen;
  }
  for (int j = 0; j < 5; j++){
    float fj = float(j);
    vec3 bp = around(0.42, a0 + (fj + 1.0)*0.07*upper, dp.y);
    float db = rayPt(ro, rd, bp, tq);
    float tw = 0.6 + 0.4*sin(uTime*13.0 + fj*2.1);
    fx += white*halo(db, 0.07)*1.6*tw*exp(-f*3.0)*(1.0 - fj*0.15)*vis;
  }
  // ---- prominences: pearly loops of plasma arching off the blade, rising and lifting away
  float dA = raySegRaw(ro, rd, vec3(0.0, 0.0, 0.0), vec3(0.0, top, 0.0), s);
  if (dA < 4.5){
    for (int i = 0; i < 4; i++){
      float fi = float(i);
      float ph = uTime*(0.25 + 0.1*h11(fi*3.3)) + h11(fi*8.1);
      float cy = floor(ph), fp = fract(ph);
      float y0 = (0.1 + 0.8*h11(cy*1.9 + fi*3.0))*top;
      if (abs(y0 - yb) < hw + 1.0) continue;
      float az = h11(cy*2.7 + fi);
      float span = 0.5 + 0.6*h11(cy + fi*5.0);
      // round 19 (Creyon: a prominence rose past the tip): each loop ends below the tip
      y0 = min(y0, top - 1.2 - span*2.0);
      float H = (0.45 + 0.5*h11(cy*4.1 + fi))*smoothstep(0.0, 0.35, fp)*(1.0 + 0.8*max(fp - 0.5, 0.0));
      float life = smoothstep(0.0, 0.1, fp)*(1.0 - smoothstep(0.7, 1.0, fp));
      vec3 prev = around(0.35, az, y0);
      for (int j = 1; j <= 10; j++){
        float u = float(j)/10.0;
        float lift = 0.0;
        vec3 p = around(0.35 + H*sin(u*3.1416) + lift, az + 0.03*sin(u*3.1416*2.0)*H, y0 + span*u*2.0);
        float dh = raySeg(ro, rd, prev, p, s);
        float strand = 0.65 + 0.35*sin((float(j) + s)*2.0 - uTime*4.0 + fi);
        fx += mix(white, pearl, 0.6)*(halo(dh, max(0.06, px))*0.8 + halo(dh, 0.3)*0.4)*life*strand;
        prev = p;
      }
    }
  }
  light += fx*uFx;
  return light;
}
// the orbiting moons' dark discs, each a disc facing the camera, in true 3D
// round 10: the eclipsed suns circle the blade, so they alone are drawn in true 3D
// (Sol's way: only what orbits is 3D, everything else flat)
vec3 base_orbit(vec3 ro, vec3 rd, float px){
  float top = ec_top(), tq;
  float cl = clamp(uClash, 0.0, 1.0);
  float flare = max(cl, exp(-uAge*1.6)*step(0.0, uAge)*0.35);
  vec3 pearl = vec3(1.0, 0.9, 0.72), white = vec3(1.0, 0.98, 0.94);
  vec3 fx = vec3(0.0);
  float vis = smoothstep(0.1, 0.6, uAge);
  // ---- the eclipsed suns orbiting the blade: a black moon rimmed by a blazing corona with
  // streamers, a diamond-ring flash, beads of light and a pearly wake behind each
  for (int i = 0; i < 3; i++){
    float fi = float(i), R;
    vec3 c = ec_orb(fi, R);
    if (c.y > top + 2.0) continue;
    vec3 f = ec_face(ro, rd, c);
    if (f.z > R*6.0) continue;
    float ang = atan(f.y, f.x);
    float rr = f.z/R;
    float strm = pow(0.5 + 0.5*sin(ang*7.0 + uTime*0.3 + fi*2.0), 3.0)*0.7 + pow(0.5 + 0.5*sin(ang*13.0 - uTime*0.5 + fi), 6.0)*0.5;
    float cor = exp(-max(rr - 1.0, 0.0)*(1.1 - 0.7*strm))*smoothstep(0.95, 1.02, rr)*(1.0 - smoothstep(3.0, 5.5, rr));
    float rim = halo(abs(rr - 1.02), 0.04 + px/R);
    float spin = uTime*0.9 + fi*2.1;
    float dia = pow(max(cos(ang - spin), 0.0), 60.0)*halo(abs(rr - 1.0), 0.15);
    fx += (pearl*cor*(1.6 + 0.8*flare) + white*rim*2.2 + white*dia*4.0 + white*halo(abs(rr - 1.0), 0.6)*pow(max(cos(ang - spin), 0.0), 8.0)*0.6)*vis;
    // beads of light circling the rim
    for (int j = 0; j < 5; j++){
      float aj = spin + 0.4 + float(j)*0.25;
      vec2 bp = vec2(cos(aj), sin(aj))*R*1.02;
      float db = length(f.xy - bp);
      fx += white*halo(db, max(0.05, px))*1.4*(0.6 + 0.4*sin(uTime*9.0 + float(j)))*vis;
    }
    // a pearly wake trailing the orbit
    for (int j = 1; j <= 6; j++){
      float lag = float(j)*0.018*(fi == 1.0 ? -1.0 : 1.0);
      float Rw;
      vec3 cw = ec_orbAt(fi, uTime - float(j)*0.25, Rw);
      float dw = rayPt(ro, rd, cw, tq);
      fx += pearl*halo(dw, 0.35*(1.0 - float(j)/7.0) + 0.05)*0.35*(1.0 - float(j)/7.0)*vis;
    }
  }
  return fx*uFx;
}
vec4 base_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 o = vec4(0.0);
  float top = ec_top();
  for (int i = 0; i < 3; i++){
    float fi = float(i), R;
    vec3 c = ec_orb(fi, R);
    if (c.y > top + 2.0) continue;
    vec3 f = ec_face(ro, rd, c);
    float a = fillD(f.z - R, px)*smoothstep(0.2, 0.6, uAge);
    o = vec4(mix(o.rgb, vec3(0.01, 0.008, 0.006), a), max(o.a, a));
  }
  return o;
}
// the moon's shadow, drawn over Sol's glow as his solid blades are: the blade's heart goes
// dark across the band
vec4 sig_solid(vec4 o, vec2 b, float px){
  float a = (1.0 - smoothstep(0.22, 0.42, abs(b.x)))*ec_shade(b.y)*smoothstep(0.1, 0.5, uAge);
  return vec4(mix(o.rgb, vec3(0.02, 0.018, 0.015), a*0.9), max(o.a, a*0.9));
}
// the blade: Sol's glow and core from the file, light running up it, out in the band
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float sheen = 0.85 + 0.15*sin(b.y*1.3 - uTime*6.0) + 0.1*sin(b.y*0.37 + uTime*2.3);
  return g*sheen*(1.0 - 0.94*ec_shade(b.y));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(1.0 - 0.94*ec_shade(b.y)); }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  
  // round 14 (Creyon): a chroma for the blade only; the corona and the suns keep their own
  // pearly colours (uKeep, uOwn), as Kyber Cluster's crystals do
  uKeep += col + fx*uFx;
  return vec3(0.0);
}
// round 11: sig_orbit gated by part 1; round 14 (Creyon): the eclipsed suns' glow takes the
// wearer's colour with the blade (their moons stay black, `sig_orbit_solid`)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5) { return vec3(0.0); }
  return base_orbit(ro, rd, px);
}
// round 11: sig_orbit_solid gated by part 1
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5) { return vec4(0.0); }
  vec4 o = base_orbit_solid(ro, rd, px);
  uOwn = max(uOwn, o.a);
  return o;
}
