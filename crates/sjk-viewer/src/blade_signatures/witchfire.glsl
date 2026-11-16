// Witchfire: the Nightsister magick of Dathomir, as a blade. The blade is a column of sickly green
// witchfire ichor with a pale burning heart, wrapped all round in swirling green ghost-fire whose
// tongues lick up its length. Thick ichor mist pours down off it and spills over the hilt, three
// tendrils of green energy coil round it and slide upward, dark smoke curls through the fire, and
// pulses of magick climb the blade from the hilt to the tip. Swinging feeds the fire; a clash
// blasts it outward in a ring of green flame (round 15: no lightning, which made it look like the
// Sith Alchemy and Superlaser blades; the ghost-fire is its own).
// @part Ghost-fire
// @wisps Ichor smoke
// @motes Nightsister spirits
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.04,0.35,0.08],"corona.rim_hot":[0.35,1.0,0.35],"corona.inner":[0.8,1.0,0.75],"core.white":[0.92,1.0,0.9],"core.fringe_cool":[0.2,0.85,0.3],"core.fringe_hot":[0.7,1.0,0.6],"light.color":[0.3,1.0,0.35],"trail":[0.35,1.0,0.4]}

const vec3 WF_G = vec3(0.10, 1.0, 0.09);
const vec3 WF_DEEP = vec3(0.02, 0.50, 0.05);
const vec3 WF_PALE = vec3(0.78, 1.0, 0.62);

float wf_top(){ float g = clamp(uAge/0.45, 0.0, 1.0); return uLength*(1.0 - (1.0 - g)*(1.0 - g)); }
// pulses of magick climbing the blade
float wf_pulse(float y){
  float f = fract(y*0.045 - uTime*0.7);
  return pow(f, 10.0)*smoothstep(1.0, 0.97, f)*2.0;
}
// one coiling tendril of green energy
vec3 wf_tendril(vec3 ro, vec3 rd, float k, float px, float top){
  float best = 1e3; float bs = 0.0; float s;
  vec3 prev = vec3(0.0);
  float ph = k/3.0 + uTime*0.12;
  float slide = uTime*0.9 + k*4.0;
  for (int i = 0; i <= 20; i++){
    float f = float(i)/20.0;
    float y = f*top*1.02 - 0.3;
    float rr = 0.9 + 0.8*(0.5 + 0.5*sin(f*7.0 + k*2.1 - uTime*1.3)) + uSwing*0.6;
    vec3 q = around(rr, ph + f*(1.3 + 0.25*k), y);
    if (i > 0){
      float d = raySeg(ro, rd, prev, q, s);
      if (d < best){ best = d; bs = (float(i) - 1.0 + s)/20.0; }
    }
    prev = q;
  }
  // energy sliding along it
  float flow = 0.45 + 0.55*pow(0.5 + 0.5*sin((bs*top - slide*6.0)*0.35), 3.0);
  float fade = smoothstep(0.0, 0.08, bs)*(1.0 - smoothstep(0.88, 1.0, bs));
  return (WF_PALE*halo(best, max(px, 0.03))*0.6 + WF_G*halo(best, 0.14)*0.8 + WF_G*halo(best, 0.6)*0.18)*flow*fade;
}
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float top = wf_top();
  float cl = clamp(uClash, 0.0, 1.0);
  vec3 col = vec3(0.0);
  // The ichor blade is Sol's glow and core from the file with pulses of magick climbing it
  // (sig_glow, sig_core); here only the effects, each a piece placed round it in 3D.
  float sa;
  float dax = raySegRaw(ro, rd, vec3(0.0, -0.6, 0.0), vec3(0.0, top, 0.0), sa);
  float hot = 1.0 + uSwing*0.6 + cl*0.8;
  // ghost-fire: green tongues licking up off the blade, swirling round it
  for (int i = 0; i < 24; i++){
    float fi = float(i);
    float rate = 0.8 + 0.5*h11(fi*2.3);
    float f = fract(uTime*rate + h11(fi*6.7));
    float k = floor(uTime*rate + h11(fi*6.7));
    float y0 = (0.02 + 0.92*h11(fi*3.1 + k))*top;
    float a0 = h11(fi*5.3 + k) + uTime*0.22;
    vec3 p0 = around(0.4, a0, y0 + 2.0*f);
    vec3 p1 = around(0.6 + (1.2 + 1.6*h11(fi*9.1))*f*hot, a0 + 0.08*f, y0 + 2.0*f + 1.3);
    float sk;
    float dk = raySeg(ro, rd, p0, p1, sk);
    if (dk > 2.0) continue;
    float w = max(0.12 + 0.25*(1.0 - sk)*(1.0 - f), px);
    float pu2 = wf_pulse(p0.y);
    col += (WF_G*halo(dk, w)*0.9*(1.0 + pu2) + WF_PALE*halo(dk, w*0.3)*0.4)*sin(3.1416*f)*(1.0 - 0.5*sk)*uFx;
  }
  // ichor mist pouring down the blade and spilling over the hilt
  for (int i = 0; i < 12; i++){
    float fi = float(i);
    float f = fract(uTime*(0.25 + 0.1*h11(fi*1.7)) + h11(fi*8.3));
    float y0 = (0.15 + 0.8*h11(fi*4.1))*top;
    float yy = mix(y0, -2.5, f);
    float sp = 1.0 - clamp(yy/uLength, 0.0, 1.0);
    vec3 p = around(0.6 + 2.2*sp*sp*f + 0.4*f, h11(fi*2.9) + 0.1*f, yy);
    float tt;
    float dm = rayPt(ro, rd, p, tt);
    col += WF_DEEP*(halo(dm, 0.5 + 0.8*f)*0.45 + halo(dm, 1.5)*0.08)*sin(3.1416*f)*uFx;
  }
  // tendrils
  if (dax < 3.4){
    vec3 td = vec3(0.0);
    for (int k = 0; k < 3; k++){ td += wf_tendril(ro, rd, float(k), px, top); }
    col += td*uFx;
  }
  // clash: a ring of green flame blasts out
  if (cl > 0.0){
    vec3 h = rayAt(ro, rd, uLength*0.6);
    if (h.z > 0.0){
      // round 23: hidden behind the blade softly (see hideD in lab.glsl)
      float rr = length(h.xy), aa = atan(h.x, h.y), seen = 1.0 - behindBlade(ro, rd, h.z);
      float R = 0.5 + (1.0 - cl)*5.0;
      float tongue = 0.6 + 0.4*vn(vec2(aa*4.0, uTime*8.0));
      col += (WF_G*halo(abs(rr - R), 0.5*tongue) + WF_PALE*halo(abs(rr - R), 0.12))*cl*1.5*uFx*seen;
    }
  }
  return col;
}
// dark smoke curling through the fire: dim puffs drifting up round it, each facing the camera
vec4 sig_vol_solid(vec3 ro, vec3 rd, float px){
  float top = wf_top();
  if (uFx < 0.5) return vec4(0.0);
  float al = 0.0;
  for (int i = 0; i < 10; i++){
    float fi = float(i);
    float f = fract(uTime*0.12 + h11(fi*3.9));
    vec3 p = around(1.5 + 1.2*h11(fi*6.1) + 0.6*f, h11(fi*2.2) + uTime*0.1, (0.1 + 0.85*h11(fi*7.7))*top + 3.0*f);
    float tt;
    float d = rayPt(ro, rd, p, tt);
    al = max(al, halo(d, 0.6 + 0.5*f)*0.45*sin(3.1416*f));
  }
  return vec4(vec3(0.01, 0.025, 0.012), al);
}
// the blade: Sol's glow and core from the file, wobbling, pulses of magick climbing it
vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  float pu = wf_pulse(b.y);
  return g*(0.85 + 0.15*sin(b.y*1.7 - uTime*6.0) + 0.8*pu + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(1.0 + 0.3*wf_pulse(b.y)); }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 3.0, vec3(0.3, 1.0, 0.35))*part(0.0);
  return col + fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 80 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.8; }
