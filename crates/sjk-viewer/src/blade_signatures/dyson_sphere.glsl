// Dyson Sphere: a captive sun harnessed by a megastructure. The blade is a burning column of
// star-stuff, a white-hot core in boiling golden plasma. Round it hangs a lattice of thin energy
// hoops at five heights, each tilted differently and turning at its own speed like a vast
// harvesting machine; pulses of collected energy race round the hoops and up thin collector
// beams from the star to each hoop, and solar flares arc out of the star in loops of plasma that
// leap up to the hoops. A swing spins the whole structure up and throws more flares; a clash
// overloads it, every hoop blazing at once.
// @part Harvester hoops
// @arcs Prominences
// @motes Rising sparks
// @room 10
// @palette {"corona.rim_cool":[0.70,0.35,0.05],"corona.rim_hot":[1.0,0.75,0.30],"corona.inner":[1.0,0.95,0.80],"core.white":[1.0,0.97,0.88],"core.fringe_cool":[1.0,0.65,0.20],"core.fringe_hot":[1.0,0.90,0.60],"light.color":[1.0,0.75,0.35],"trail":[1.0,0.78,0.4]}

float ds_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
// hoop k: centre height, radius, tilt, how fast its tilt turns round
float ds_hy(float k){ return uLength*(0.14 + 0.18*k) + 0.4*sin(uTime*0.5 + k*1.7); }
float ds_hr(float k){ return 2.2 + 2.8*sin(3.1416*(0.1 + 0.2*k)); }
float ds_tilt(float k){ return 0.42*sin(k*2.3 + 0.6) + 0.12*sin(uTime*0.4 + k); }
float ds_prec(float k){ return uTime*(0.25 + 0.12*k)*(mod(k, 2.0) < 0.5 ? 1.0 : -1.0) + k*1.3; }
// a point on hoop k at angle th (radians), with the hoop grown by g
vec3 ds_pt(float k, float th, float g){
  float tl = ds_tilt(k), pr = ds_prec(k);
  vec3 n = vec3(sin(tl)*cos(pr), cos(tl), sin(tl)*sin(pr));
  vec3 u = normalize(cross(n, vec3(0.0, 0.0, 1.0)) + vec3(0.0, 0.0, 1e-4));
  vec3 v = cross(n, u);
  return vec3(0.0, ds_hy(k), 0.0) + (u*cos(th) + v*sin(th))*ds_hr(k)*g;
}

vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, top = ds_top(), s, tq;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 gold = vec3(1.0, 0.72, 0.3), white = vec3(1.0, 0.96, 0.86);
  // The star column is Sol's glow and core from the file (sig_glow, sig_core); here only
  // the effects, each a piece placed round it in 3D.
  vec3 light = vec3(0.0);
  vec3 fx = vec3(0.0);
  // tongues of flame licking up the column
  for (int i = 0; i < 16; i++){
    float fi = float(i);
    float f = fract(uTime*(0.7 + 0.4*h11(fi*2.1)) + h11(fi*6.3));
    float cy = floor(uTime*(0.7 + 0.4*h11(fi*2.1)) + h11(fi*6.3));
    float y0 = (0.03 + 0.9*h11(fi*3.9 + cy))*top;
    float a0 = h11(fi*1.7 + cy);
    vec3 p0 = around(0.5, a0, y0 + 1.2*f);
    vec3 p1 = around(0.6 + 0.7*f, a0 + 0.04, y0 + 1.2*f + 0.9);
    float dd = raySeg(ro, rd, p0, p1, s);
    fx += mix(white, gold, f)*(halo(dd, max(0.08*(1.0 - s) + 0.03, px))*1.3 + gold*halo(dd, 0.35)*0.3)*sin(3.1416*f);
  }
  // ---- the hoops: thin rings of light, each with pulses of energy racing round it
  for (int k = 0; k < 5; k++){
    float fk = float(k);
    float hy = ds_hy(fk), R = ds_hr(fk);
    float g = smoothstep(hy - 0.5, hy + 3.0, top);
    if (g < 0.01) continue;
    float dc = rayPtRaw(ro, rd, vec3(0.0, hy, 0.0), tq);
    if (dc > R*g + 1.2) continue;
    float flow = (mod(fk, 2.0) < 0.5 ? 1.0 : -1.0)*uTime*(2.0);
    vec3 prev = ds_pt(fk, 0.0, g);
    float best = 1e3, bth = 0.0;
    for (int i = 1; i <= 20; i++){
      float th = float(i)*0.31416;
      vec3 p = ds_pt(fk, th, g);
      float dh = raySeg(ro, rd, prev, p, s);
      if (dh < best){ best = dh; bth = th - 0.31416*(1.0 - s); }
      prev = p;
    }
    float pulse = 0.35 + 0.65*pow(0.5 + 0.5*sin(bth*3.0 - flow*2.0), 6.0) + 0.3*pow(0.5 + 0.5*sin(bth*7.0 + flow*1.3), 8.0);
    float hot = 1.0 + 2.0*cl;
    fx += (white*halo(best, max(0.04, px))*1.4*pulse + gold*halo(best, 0.22)*0.45*(0.5 + 0.5*pulse) + gold*halo(best, 0.8)*0.07)*g*hot;
    // collector beams: thin rays from the star out to the hoop, energy flowing out along them
    for (int j = 0; j < 3; j++){
      float th = float(j)*2.0944 + fk*0.7 + uTime*0.3;
      vec3 p = ds_pt(fk, th, g);
      vec3 a = normalize(vec3(p.x, 0.0, p.z))*0.5 + vec3(0.0, p.y - 1.5, 0.0);
      float db = raySeg(ro, rd, a, p, s);
      float fl = 0.08 + 0.92*pow(0.5 + 0.5*sin(s*9.0 - uTime*(10.0) + fk + float(j)*2.0), 6.0);
      fx += (white*halo(db, max(0.03, px))*1.1 + gold*halo(db, 0.18)*0.35)*fl*g*(1.0 - 0.5*s)*hot;
    }
  }
  // ---- solar flares: loops of plasma leaping from the star up to a hoop and falling back
  for (int i = 0; i < 3; i++){
    float fi = float(i);
    float ph = uTime*(0.45 + 0.15*fi) + h11(fi*5.3);
    float cy = floor(ph), f = fract(ph);
    float k = floor(h11(cy*2.9 + fi)*4.99);
    float hy = ds_hy(k);
    if (hy > top - 1.0) continue;
    float th = 6.2832*h11(cy*1.7 + fi*3.1);
    vec3 b = ds_pt(k, th, 1.0);
    vec3 a = vec3(0.0, hy - 2.5 - 2.0*h11(cy*3.3 + fi), 0.0) + normalize(vec3(b.x, 0.0, b.z))*0.45;
    vec3 mid = (a + b)*0.5 + normalize(vec3(b.x, 0.0, b.z))*(1.2 + 1.0*h11(cy + fi)) + vec3(0.0, -0.3, 0.0);
    float grow = smoothstep(0.0, 0.45, f), life = 1.0 - smoothstep(0.65, 1.0, f);
    vec3 prev = a;
    for (int j = 1; j <= 10; j++){
      float u = float(j)/10.0;
      if (u - 0.1 > grow) break;
      vec3 p = mix(mix(a, mid, u), mix(mid, b, u), u);
      float df = raySeg(ro, rd, prev, p, s);
      float uu = u - 0.1*(1.0 - s);
      float tipf = 1.0 - smoothstep(grow - 0.1, grow, uu);
      float knot = 0.7 + 0.3*sin(uu*20.0 - uTime*14.0);
      fx += mix(white, gold, 0.4)*(halo(df, max(0.09, px))*1.6*knot + halo(df, 0.45)*0.45)*life*mix(1.0, tipf, step(grow, 0.99));
      prev = p;
    }
  }
  light += fx*uFx;
  return light;
}
// the star: Sol's glow and core from the file, boiling
vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  return g*(0.8 + 0.2*sin(b.y*1.1 - uTime*5.0) + 0.15*sin(b.y*0.31 + uTime*1.7) + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 3.2, vec3(1.0, 0.7, 0.3))*part(0.0);
  return col + fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 65 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.65; }
