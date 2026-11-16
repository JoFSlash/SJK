// Wraithlight: a pale haunted blade, like the light of a Force ghost. The blade is translucent and
// cold blue-white, brighter at its edges than its heart, like a glass rod full of moonlight, and
// it shimmers as slow ripples of light pass up it. Cold spirit mist pours off it and sinks toward
// the hilt in soft drifting veils, and faint echoes of the blade itself, its afterglows, hang in
// the air round it, drifting slowly, fading in and out of sight. Swinging leaves more echoes
// behind; a clash makes the blade flare and blows the mist out in a soft ring. Round 15b (Creyon:
// mythicals must not look cheap): three wraiths, pale lights with long ghostly tails, dart round
// and through the blade on looping paths, and its cold glow breathes.
// @part Mist and echoes
// @wisps Spirit mist
// @motes Souls
// @room 10
// @palette {"corona.rim_cool":[0.30,0.55,0.95],"corona.rim_hot":[0.72,0.86,1.0],"corona.inner":[0.88,0.95,1.0],"core.white":[0.92,0.97,1.0],"core.fringe_cool":[0.45,0.68,1.0],"core.fringe_hot":[0.78,0.90,1.0],"light.color":[0.55,0.75,1.0],"trail":[0.55,0.75,1.0]}

const vec3 WL_COL = vec3(0.42, 0.66, 1.0);
const vec3 WL_PALE = vec3(0.82, 0.92, 1.0);

float wl_top(){ float g = clamp(uAge/0.45, 0.0, 1.0); return uLength*(1.0 - (1.0 - g)*(1.0 - g)); }
// one echo of the blade: a faint copy drifting round it, fading in and out
vec3 wl_echo(vec3 ro, vec3 rd, float i, float px, float top){
  float per = 7.0 + 2.0*h11(i*3.1);
  float ph = fract(uTime/per + h11(i*7.7));
  float life = sin(ph*3.1416); life = life*life;
  float rad = 1.1 + 1.5*h11(i*2.3) + ph*0.9 + uSwing*1.2;
  float ang = h11(i*5.9) + uTime*0.025*(h11(i) > 0.5 ? 1.0 : -1.0) + ph*0.08;
  float y0 = 1.0 + 6.0*h11(i*4.4) + ph*1.5;
  float len = top*(0.5 + 0.35*h11(i*9.1));
  vec3 a = around(rad, ang, y0);
  vec3 b = around(rad + 0.6*(h11(i*1.9) - 0.5), ang + 0.03*(h11(i*6.1) - 0.5), y0 + len);
  float s; float d = raySeg(ro, rd, a, b, s);
  float ends = smoothstep(0.0, 0.25, s)*(1.0 - smoothstep(0.7, 1.0, s));
  float flick = 0.75 + 0.25*sin(uTime*(2.0 + h11(i)*3.0) + s*9.0 - uTime*2.0);
  return (WL_PALE*halo(d, max(px*0.8, 0.07))*0.45 + WL_COL*halo(d, 0.35)*0.22 + WL_COL*halo(d, 1.0)*0.05)*ends*life*flick*(0.9 + 0.7*uSwing);
}
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float top = wl_top();
  float cl = clamp(uClash, 0.0, 1.0);
  // The blade is Sol's glow and core from the file, hollowed to a glass rod of moonlight
  // (sig_glow, sig_core); here only the effects, each a piece placed round it in 3D.
  float sa;
  float dax = raySegRaw(ro, rd, vec3(0.0, -0.6, 0.0), vec3(0.0, top, 0.0), sa);
  vec3 col = vec3(0.0);
  // spirit mist pouring off it and sinking toward the hilt in soft veils
  for (int i = 0; i < 16; i++){
    float fi = float(i);
    float f = fract(uTime*(0.12 + 0.06*h11(fi*1.3)) + h11(fi*5.1));
    float y0 = (0.2 + 0.78*h11(fi*2.7))*top;
    float yy = mix(y0, -2.0, f);
    float down = clamp(1.0 - yy/uLength, 0.0, 1.0);
    vec3 p = around(0.6 + (1.0 + 1.6*down)*f + 1.6*cl, h11(fi*8.3) + 0.05*f, yy);
    float tt;
    float dm = rayPt(ro, rd, p, tt);
    if (dm > 3.5) continue;
    col += mix(WL_COL, WL_PALE, 0.3)*(halo(dm, 0.6 + 0.9*f)*0.35 + halo(dm, 1.8)*0.06)*sin(3.1416*f)*uFx;
  }
  // echoes of the blade
  if (dax < 5.0){
    vec3 e = vec3(0.0);
    for (int i = 0; i < 6; i++){ e += wl_echo(ro, rd, float(i), px, top); }
    col += e*uFx;
  }
  // clash: a soft ring of mist blown out
  if (cl > 0.0){
    vec3 h = rayAt(ro, rd, uLength*0.55);
    if (h.z > 0.0){
      // round 23: hidden behind the blade softly (see hideD in lab.glsl)
      float rr = length(h.xy), aa = atan(h.x, h.y), seen = 1.0 - behindBlade(ro, rd, h.z);
      float R = 0.6 + (1.0 - cl)*4.5;
      float n = 0.6 + 0.4*vn(vec2(aa*3.0, rr*2.0));
      col += WL_PALE*halo(abs(rr - R), 0.6)*cl*n*0.9*uFx*seen;
    }
  }
  return col;
}
// the blade: Sol's glow and core from the file as a translucent glass rod: brighter at its rim
// than its heart, slow ripples of light passing up it, flaring on a clash
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float rip = 0.5 + 0.5*sin(b.y*0.45 - uTime*2.2);
  rip = rip*rip;
  float shim = 0.85 + 0.15*sin(b.y*2.3 + uTime*7.0)*sin(b.y*0.7 - uTime*3.1);
  float R = 0.42 + 0.04*sin(b.y*1.1 - uTime*2.6);
  float rim = exp(-pow((abs(b.x) - R)/(R*0.35 + px), 2.0))*smoothstep(-0.6, 0.2, b.y)*(1.0 - smoothstep(wl_top() - 1.0, wl_top(), b.y));
  float hollow = 1.0 - 0.45*exp(-pow(b.x/(R*0.7), 2.0));
  // round 12 (Creyon: still too shiny): the whole blade calmer, the rim fainter
  return (g*hollow*0.55 + WL_PALE*rim*0.2*(0.8 + 0.6*rip))*shim*(1.0 + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*0.6; }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 2.2, vec3(0.55, 0.75, 1.0))*0.35*part(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 1.5, 6.0, 24.0, 0.04, 2.9, vec3(0.85, 0.93, 1.0))*part(0.0);
  // round 16 (Creyon: still missing an effect): cold spirit fire, pale ghost-flames licking up
  // the blade fast and thin, and frost glittering where they pass
  fx += fxSmoke(ro, rd, y0, y1, 1.3, 7.5, 5.7, vec3(0.55, 0.8, 1.0))*0.9*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 0.7, 9.0, 1.3, vec3(0.85, 0.95, 1.0))*0.6*part(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 2.2, 10.0, -1.2, 0.12, 0.9, 3.0, 0.3, 4.4, vec3(0.85, 0.95, 1.0))*0.6*part(0.0);
  return col + fx*uFx;
}

// round 15b: three wraiths darting round and through the blade on looping paths (figure eights
// tilted round it), each a pale head trailing a long ghostly tail; they circle the blade, so
// they are drawn in 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  // with the mist and echoes (part 0): it has six options already
  if (part(0.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, L*0.1, L) > 6.0) return fx;
  for (int j = 0; j < 3; j++){
    float fj = float(j);
    float best = 1e3, bs = 0.0;
    vec3 prev = vec3(0.0);
    for (int i = 0; i <= 16; i++){
      float tt = t*0.55 - float(i)*0.05 + fj*2.1;
      float a = tt*0.16 + fj/3.0;
      float r = 2.6*sin(tt*0.9 + fj);
      vec3 p = around(abs(r) + 0.3, a + (r < 0.0 ? 0.5 : 0.0), L*(0.5 + 0.35*sin(tt*0.5 + fj*1.3)));
      if (i > 0 && length(p - prev) < 3.0){
        float d = raySeg(ro, rd, prev, p, s);
        if (d < best){ best = d; bs = (float(i) - 1.0 + s)/16.0; }
      }
      prev = p;
    }
    float tail = 1.0 - bs;
    vec3 cold = vec3(0.6, 0.8, 1.0);
    fx += (vec3(1.0)*halo(best, max(0.08, px))*pow(tail, 8.0)*2.2 + cold*halo(best, 0.25 + 0.3*bs)*tail*0.8
           + cold*halo(best, 0.9)*0.12*tail);
  }
  return fx*uFx;
}
