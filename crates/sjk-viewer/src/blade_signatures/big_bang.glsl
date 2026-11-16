// Big Bang: a blade that holds a young universe. Each ignition starts as one blinding point at the
// emitter that explodes: a sphere of light bursts outward and a fireball races up and lights the
// blade in half a second. Then the blade is a living cosmos: a white-hot core, shells of light
// rippling outward from it, soft nebula clouds swirling round it in three dimensions and a field of
// stars turning about it and drifting outward as space expands. Every so often a small new bang
// goes off somewhere along the blade, a spherical shockwave racing out. A swing stirs the gas and
// brightens the stars; a clash sets off a big bang flash.
// @part Young cosmos
// @part New stars
// @motes Expanding stars
// @hue Cosmic colours
// @room 10
// @palette {"corona.rim_cool":[0.30,0.08,0.45],"corona.rim_hot":[0.95,0.45,0.75],"corona.inner":[1.0,0.85,0.92],"core.white":[1.0,0.97,0.95],"core.fringe_cool":[0.70,0.40,1.0],"core.fringe_hot":[1.0,0.70,0.80],"light.color":[0.95,0.55,0.85],"trail":[0.90,0.50,0.85]}


// the blade lights in half a second behind a racing fireball
float bb_top(){ return uLength*smoothstep(0.04, 0.48, uAge); }
// a spherical shell of light (the ray's closest pass to its centre shows its bright rim)
float bb_shell(vec3 ro, vec3 rd, vec3 c, float R, float w, out float dc){
  float t;
  dc = rayPt(ro, rd, c, t);
  return halo(abs(dc - R), w) + 0.35*smoothstep(max(R, 0.02), 0.0, dc);
}

// the ray's distance to a ring of radius R round the blade at height y
float bb_ring(vec3 ro, vec3 rd, float y, float R){
  float best = 1e3;
  vec3 h = rayAt(ro, rd, y);
  if (h.z > 0.0) best = hideD(abs(length(h.xy) - R), behindBlade(ro, rd, h.z));
  float a = dot(rd.xz, rd.xz), b = dot(ro.xz, rd.xz), c = dot(ro.xz, ro.xz) - R*R;
  float dsc = b*b - a*c;
  if (dsc > 0.0 && a > 1e-6){
    float q = sqrt(dsc);
    float t0 = (-b - q)/a, t1 = (-b + q)/a;
    if (t0 > 0.0) best = min(best, hideD(abs(ro.y + rd.y*t0 - y), behindBlade(ro, rd, t0)));
    if (t1 > 0.0) best = min(best, hideD(abs(ro.y + rd.y*t1 - y), behindBlade(ro, rd, t1)));
  }
  return best;
}

vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, top = bb_top(), age = uAge;
  float st = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0), fx = uFx;
  vec3 white = vec3(1.0, 0.97, 0.95), rose = vec3(1.0, 0.35, 0.72), violet = vec3(0.55, 0.3, 1.0), gold = vec3(1.0, 0.75, 0.45);
  float young = exp(-age*2.5);
  vec3 col = vec3(0.0);
  float s, ta;

  // ---- the singularity and the first burst ----
  if (age < 1.2){
    vec3 O = vec3(0.0, 0.4, 0.0);
    float dO;
    float pre = smoothstep(0.0, 0.05, age)*(1.0 - smoothstep(0.06, 0.12, age));
    float burst = smoothstep(0.05, 0.08, age)*exp(-max(age - 0.06, 0.0)*3.5);
    // the burst: an egg of light swelling out round the blade and racing up it with the fireball
    float ex = max(age - 0.05, 0.0);
    float Rh = 0.05 + 5.0*(1.0 - exp(-ex*7.0)), Ry = max(top + 2.0, Rh);
    vec3 sc = vec3(1.0, Rh/Ry, 1.0);
    vec3 rd2 = normalize(rd*sc);
    float tt;
    float dS = rayPt(ro*sc - O*sc, rd2, vec3(0.0), tt);
    float sh = halo(abs(dS - Rh), 0.3 + 0.25*Rh/5.0) + 0.35*smoothstep(Rh, 0.0, dS);
    dO = rayPt(ro, rd, O, ta);
    col += white*(halo(dO, 0.35)*pre*6.0 + exp(-dO/1.5)*pre*1.5);
    col += mix(white, rose, smoothstep(0.0, 0.8, age))*sh*burst*1.3*fx;
    // the fireball racing up the blade
    float fb = rayPtRaw(ro, rd, vec3(0.0, top, 0.0), ta);
    col += mix(white, gold, 0.3)*(halo(fb, 0.8)*2.0 + exp(-fb/2.0)*0.6)*(1.0 - smoothstep(0.4, 0.6, age))*step(0.05, age);
  }
  if (top < 0.1) return col;

  // The blade itself is Sol's glow and core from the skin's file (sig_glow, sig_core).
  float far = raySegRaw(ro, rd, vec3(0.0, -1.0, 0.0), vec3(0.0, top + 1.0, 0.0), s);
  if (fx <= 0.0 || far > 9.0) return col;

  // ---- nebula clouds: soft puffs of gas turning round the blade, inner ones faster ----
  for (int i = 0; i < 16; i++){
    float fi = float(i);
    float r = 1.0 + 3.2*h11(fi*5.3);
    float yy = (0.05 + 0.9*h11(fi*2.7))*L;
    if (yy > top + 1.0) continue;
    vec3 p = around(r, h11(fi*1.9) + t*0.1/(0.6 + 0.3*r), yy + 1.2*sin(t*0.25 + fi*1.3));
    float dd = rayPt(ro, rd, p, ta);
    if (dd > 4.0) continue;
    float k = h11(fi*8.1);
    vec3 gas = k < 0.33 ? vec3(0.3, 0.35, 1.0) : k < 0.66 ? violet : rose;
    float br = 0.55 + 0.45*sin(t*0.9 + fi*2.1);
    col += gas*(halo(dd, 0.7 + 0.6*h11(fi*3.3))*0.7 + halo(dd, 1.8)*0.16)*br*(1.0 + 0.5*st + 1.5*young);
  }
  // ---- shells of light rippling outward from the blade, as rings round it ----
  for (int k = 0; k < 3; k++){
    float f = fract(t*0.28 + float(k)/3.0);
    float yr = (0.2 + 0.6*h11(float(k)*4.1 + floor(t*0.28 + float(k)/3.0)))*L;
    if (yr > top) continue;
    float R = 0.6 + 5.0*f;
    float dr = bb_ring(ro, rd, yr, R);
    col += mix(white, rose, 0.4)*(halo(dr, max(0.05, px))*1.2 + halo(dr, 0.4)*0.25)*(1.0 - f)*(1.0 - f)*smoothstep(0.0, 0.08, f);
  }

  // ---- stars turning round the blade and drifting outward as space expands ----
  for (int i = 0; i < 40; i++){
    float fi = float(i);
    float e = fract(h11(fi*3.7) + t*0.025);
    float r = 0.7 + 4.4*e;
    float yy = h11(fi*1.31)*(L + 1.0);
    if (yy > top) continue;
    vec3 p = around(r, h11(fi*7.9) + t*0.12/(0.5 + 0.35*r), yy + 0.4*sin(t*0.3 + fi));
    float dd = rayPt(ro, rd, p, ta);
    float tw = 0.65 + 0.35*sin(t*(3.0 + 3.0*h11(fi)) + fi*1.7);
    float life = smoothstep(0.0, 0.12, e)*(1.0 - smoothstep(0.75, 1.0, e));
    vec3 sc = mix(white, h11(fi*2.3) < 0.5 ? vec3(0.75, 0.82, 1.0) : vec3(1.0, 0.85, 0.7), 0.5);
    col += sc*(halo(dd, max(0.06, px))*2.2 + halo(dd, 0.28)*0.18)*tw*life*(1.0 + 0.6*st);
  }

  // ---- mini-bangs: a flash and a spherical shockwave somewhere along the blade ----
  for (int i = 0; i < 2; i++){
    float fi = float(i);
    float cyc = t/2.6 + fi*0.5;
    float k = floor(cyc), q = fract(cyc)*2.6;
    float yb = (0.15 + 0.75*h11(k*3.3 + fi*7.1))*L;
    if (yb > top) continue;
    vec3 c = vec3(0.0, yb, 0.0);
    float dc;
    float R = 4.8*(1.0 - exp(-q*2.2));
    float sh = bb_shell(ro, rd, c, R, 0.12 + 0.12*R, dc);
    float fade = exp(-q*1.4)*smoothstep(0.0, 0.05, q);
    col += (mix(white, rose, smoothstep(0.0, 1.2, q))*sh*1.2 + white*halo(dc, 0.6)*exp(-q*6.0)*2.0)*fade;
  }

  // ---- clash: a big bang ----
  if (cl > 0.01){
    float a = 1.0 - cl;
    float dc;
    float sh = bb_shell(ro, rd, vec3(0.0, min(L*0.6, top), 0.0), 0.5 + 5.5*(1.0 - exp(-a*4.0)), 0.5, dc);
    col += white*(halo(dc, 1.8)*3.5 + halo(dc, 3.4)*1.2)*cl*cl + mix(white, rose, a)*sh*cl*1.4;
  }
  return col;
}

// the blade: Sol's glow and core from the file, heated where a small bang goes off
float bb_mb(float y){
  float mb = 0.0;
  for (int i = 0; i < 2; i++){
    float fi = float(i);
    float cyc = uTime/2.6 + fi*0.5;
    float k = floor(cyc), q = fract(cyc)*2.6;
    float yb = (0.15 + 0.75*h11(k*3.3 + fi*7.1))*uLength;
    mb += exp(-pow((y - yb)/1.4, 2.0))*exp(-q*3.0);
  }
  return mb + exp(-uAge*2.5) + 0.5*clamp(uClash, 0.0, 1.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return g*(0.9 + 0.1*sin(uTime*1.7 - b.y*0.35) + bb_mb(b.y) + 0.3*uSwing); }
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(1.0 + 0.6*bb_mb(b.y)); }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 pink = vec3(1.0, 0.45, 0.85), blue = vec3(0.45, 0.6, 1.0);
  for (int i = 0; i < 4; i++){
    float fi = float(i);
    float cyc = t*0.35 + fi*0.25, k = floor(cyc), q = fract(cyc);
    vec3 p = around(1.5 + 4.0*h11(k*3.1 + fi), h11(k*1.7 + fi*2.0), mix(y0, y1, h11(k*5.3 + fi)));
    fx += fxFlare(ro, rd, p, px, 0.18, 2.8, 6.0, t*0.3 + fi, mix(pink, blue, h11(k + fi)))*sin(3.1416*q)*part(1.0);
  }
  
  return col + fx*uFx;
}
