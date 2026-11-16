// Singing Kyber: a kyber crystal that sings, as kyber does. The blade is a long six-sided
// crystal of teal light with a white-hot heart: its faces and edges catch the light and glint
// as you turn it, and fracture planes deep inside it flash. It holds a standing note: bright
// bands sit still along its length and tremble, and between them a shimmering harmonic aura
// swells and shrinks with the wave. Sound rings leave the bands and travel up the blade,
// spreading outward and fading. A swing raises the pitch (more bands, faster rings, a brighter
// aura); a clash is a sonic boom: a shockwave sphere bursts out from the blade.
// @part Crystal song
// @part Sound rings
// @star Singing glints
// @motes Harmonics
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.11,0.50,0.47],"corona.rim_hot":[0.2,0.9,0.85],"corona.inner":[0.9,1.0,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.12,0.54,0.51],"core.fringe_hot":[0.2,0.9,0.85],"light.color":[0.2,0.9,0.85],"trail":[0.2,0.9,0.85]}


// a flattened shell of sound: the ray's closest distance to centre c with heights squeezed by k
// (so the shell is a lens k times wider than tall), and the light of a shell of radius R there
float sk_shell(vec3 ro, vec3 rd, vec3 c, float R, float k, float w){
  vec3 o = ro - c; o.y *= k;
  vec3 d = rd; d.y *= k; d = normalize(d);
  float tt = max(dot(-o, d), 0.0);
  float r = length(o + d*tt);
  float inside = clamp(w/sqrt(max(R*R - r*r, w*w)), 0.0, 1.0)*smoothstep(R + w, R - 2.0*w, r);
  return inside + halo(abs(r - R), w*1.5)*0.35;
}
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 teal = vec3(0.2, 0.92, 0.86), pale = vec3(0.85, 1.0, 1.0), white = vec3(1.0);
  float top = L*clamp(uAge/0.45, 0.0, 1.0);
  float fx = uFx;
  // the standing wave: nodes (bright bands) along the blade
  float N = 7.0 + 2.0*sw;
  float vib = 0.75 + 0.25*sin(t*41.0) + 0.15*sin(t*67.0);
  // The crystal is Sol's glow and core from the file with a kyber crystal's facets over it
  // (sig_glow, sig_solid); here only the effects, each a piece placed round it in 3D.
  vec3 light = vec3(0.0);
  float s;
  if (fx > 0.0){
    vec3 fxl = vec3(0.0);
    float near = raySegRaw(ro, rd, vec3(0.0, 0.0, 0.0), vec3(0.0, top, 0.0), s);
    if (near < 6.5){
      // ---- the standing-wave nodes: bright bands that tremble, flat glows round the crystal
      for (int i = 0; i < 9; i++){
        float fi = float(i);
        if (fi >= N) continue;
        float y = (fi + 0.5)*L/N;
        if (y > top) continue;
        float sh = sk_shell(ro, rd, vec3(0.0, y, 0.0), 0.95 + 0.15*sin(t*41.0 + fi*1.7), 4.0, 0.14);
        float to;
        float dn = rayPtRaw(ro, rd, vec3(0.0, y, 0.0), to);
        fxl += (pale*sh*0.8 + teal*halo(dn, 1.3)*0.5)*vib;
      }
      // ---- sound rings travelling up the blade and spreading outward
      for (int i = 0; i < 8; i++){
        float fi = float(i);
        float ph = t*(0.55) + fi/8.0;
        float f = fract(ph);
        float cyc = floor(ph);
        float k = floor(h11(cyc*1.7 + fi*3.1)*N);
        float y0 = (k + 0.5)*L/N;
        float y = y0 + f*7.0;
        float R = 0.8 + f*5.0;
        if (y > top + 1.0) continue;
        float w = 0.12 + 0.35*f;
        float amp = (1.0 - f)*(1.0 - f)*smoothstep(0.0, 0.1, f)*(1.0 + 0.6*sw);
        float sh = sk_shell(ro, rd, vec3(0.0, y, 0.0), R, 2.2, w);
        fxl += mix(pale, teal, 0.3 + 0.5*f)*sh*amp*1.1;
      }
    }
    // ---- a clash: the sonic boom, a shockwave sphere bursting out
    if (cl > 0.01){
      vec3 o = vec3(0.0, L*0.6, 0.0);
      float to;
      float dc = rayPt(ro, rd, o, to);
      float R = 1.8 + 5.0*(1.0 - cl);
      float sw2 = 0.35 + 0.5*(1.0 - cl);
      float shell = sk_shell(ro, rd, o, R, 1.3, sw2);
      fxl += (pale*shell*1.1 + teal*shell*0.6)*cl + pale*halo(dc, 3.0)*pow(cl, 3.0)*0.35;
    }
    light += fxl*fx;
  }
  return light;
}
float sk_node(float y){
  float N = 7.0 + 2.0*clamp(uSwing, 0.0, 1.0);
  return pow(abs(cos(3.14159*N*clamp(y, 0.0, uLength)/uLength)), 6.0);
}
float sk_vib(){ return 0.75 + 0.25*sin(uTime*41.0) + 0.15*sin(uTime*67.0); }
// the blade: Sol's glow and core from the file, its standing wave's nodes blazing and trembling
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g*(0.8 + 0.9*sk_node(b.y)*sk_vib() + 0.4*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(0.9 + 0.3*sk_node(b.y)); }
// the kyber crystal over the glow, as Sol's solid blades are drawn: a clear hexagonal prism
// (three faces seen) with bright edges, a glint sweeping round it, fractures flashing inside
vec4 sig_solid(vec4 o, vec2 b, float px){
  float top = uLength*clamp(uAge/0.45, 0.0, 1.0), t = uTime;
  float ap = 0.42;
  float w = ap*(1.0 - smoothstep(top - 1.6, top, b.y));
  float cov = fillD(abs(b.x) - w, px)*smoothstep(-0.2, 0.3, b.y)*step(0.02, w);
  if (cov <= 0.0) return o;
  float u = b.x/max(w, 0.01);
  // two face edges at a half of the width, turning slowly with the glint
  float e1 = halo(abs(u - 0.5), 0.05 + px) + halo(abs(u + 0.5), 0.05 + px) + halo(1.0 - abs(u), 0.08 + px);
  float face = u < -0.5 ? 0.0 : (u < 0.5 ? 1.0 : 2.0);
  float sweep = pow(0.5 + 0.5*cos(face*1.0472 - t*1.7 - b.y*0.12), 18.0);
  float fr = pow(0.5 + 0.5*sin(b.y*1.6 + u*2.0 + t*0.8), 30.0) + pow(0.5 + 0.5*sin(-b.y*1.3 + u*3.0 - t*0.6), 30.0);
  vec3 teal = vec3(0.2, 0.92, 0.86), pale = vec3(0.85, 1.0, 1.0);
  vec3 c = teal*0.25 + pale*fr*0.9 + vec3(1.0)*e1*(0.5 + 1.2*sweep) + pale*sweep*0.4;
  float al = cov*clamp(0.18 + 0.5*e1 + 0.4*sweep + 0.3*fr, 0.0, 0.85);
  return vec4(mix(o.rgb, c, al), max(o.a, al));
}

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 teal = vec3(0.2, 0.95, 0.85);
  for (int i = 0; i < 4; i++){
    float fi = float(i);
    float q = fract(t*0.5 + fi/4.0);
    fx += fxShock(ro, rd, px, mix(L*0.25, L*0.9, fract(fi*0.37 + floor(t*0.5 + fi/4.0)*0.61)), q, 6.0, teal)*0.9*part(1.0);
  }
  return col + fx*uFx;
}
