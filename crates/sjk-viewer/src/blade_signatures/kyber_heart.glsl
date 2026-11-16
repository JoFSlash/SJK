// Kyber Heart: a kyber crystal blade that beats like a living heart. The blade is rose-red
// crystal light with a white-hot core, and veins of light wind round it. On every heartbeat,
// lub-dub, a double pulse of light pumps up from the hilt through the veins to the tip: as the
// pulse passes, the blade swells and blazes, and each beat sends a soft glow rolling
// outward from the base of the blade. Between beats the blade dims and rests. Three arteries
// of light wind round it, lifeblood surging up them, and sparks crackle off them. A swing makes
// every beat pump harder and brighter; a clash makes the heart skip: a white flare floods the
// whole blade at once and a strong ring bursts out.
// @part Heart veins
// @part Heartbeat glow
// @veins Kyber veins
// @motes Kyber motes
// @room 10
// @palette {"corona.rim_cool":[0.45,0.04,0.18],"corona.rim_hot":[1.0,0.30,0.50],"corona.inner":[1.0,0.80,0.88],"core.white":[1.0,0.94,0.96],"core.fringe_cool":[0.90,0.20,0.45],"core.fringe_hot":[1.0,0.65,0.78],"light.color":[1.0,0.30,0.50],"trail":[1.0,0.32,0.52]}


const float KH_RATE = 1.2;   // beats a second (steady: a changing rate makes the phase race)
const float KH_V = 52.0;     // how fast a pulse climbs the blade (units a second)
// the double beat at phase ph of a beat: lub, then a softer dub
float kh_beatAt(float ph){
  return exp(-pow(ph/0.07, 2.0)) + exp(-pow((ph - 1.0)/0.07, 2.0))
       + 0.65*exp(-pow((ph - 0.24)/0.07, 2.0));
}
// the pulse at height y (it left the hilt y/KH_V seconds ago)
float kh_pulse(float y){
  float ph = fract(uTime*KH_RATE - max(y, 0.0)/KH_V*KH_RATE);
  return kh_beatAt(ph);
}

vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 rose = vec3(1.0, 0.28, 0.48), deep = vec3(0.75, 0.05, 0.2), pale = vec3(1.0, 0.78, 0.86), white = vec3(1.0);
  float top = L*clamp(uAge/0.45, 0.0, 1.0);
  float fx = uFx;
  float pump = 1.0 + 0.6*sw;
  // The blade is Sol's glow and core from the file, swelling with each pulse (sig_glow,
  // sig_core); here only the effects, each a piece placed round it in 3D.
  float s;
  vec3 light = vec3(0.0);
  if (fx > 0.0){
    vec3 fxl = vec3(0.0);
    float near = raySegRaw(ro, rd, vec3(0.0, 0.0, 0.0), vec3(0.0, top, 0.0), s);
    // ---- the veins: four strands of light wound round the blade, two each way
    if (near < 1.8){
      for (int v = 0; v < 4; v++){
        float fv = float(v);
        float dir = v < 2 ? 1.0 : -1.0;
        float ph0 = fv*0.5 + (v < 2 ? 0.0 : 0.25);
        vec3 prev = vec3(0.0, -0.4, 0.0);
        float best = 1e3, by = 0.0;
        for (int k = 1; k <= 16; k++){
          float f = float(k)/16.0;
          float yy = mix(-0.4, top, f);
          float pk = kh_pulse(yy)*pump + cl;
          float r = (0.5 + 0.07*sin(yy*0.35 + fv*2.0) + 0.06*pk)*smoothstep(-0.4, 1.5, yy)*(1.0 - smoothstep(top - 2.5, top, yy)*0.8);
          vec3 q = around(r, ph0 + dir*yy*0.036, yy);
          float sv;
          float dv = raySeg(ro, rd, prev, q, sv);
          if (dv < best){ best = dv; by = mix(prev.y, q.y, sv); }
          prev = q;
        }
        float pv = kh_pulse(by)*pump + cl*1.2;
        fxl += (mix(rose, white, min(pv, 1.0))*halo(best, max(px, 0.03))*(0.35 + 1.3*pv) + rose*halo(best, 0.16)*(0.35 + 1.0*pv))*smoothstep(-0.4, 0.5, by);
      }
    }
    // (round 13, Creyon: no rings round the blade: the beat shows in the pulse, the glow and the sparks)
    // ---- each beat throws a spray of rose sparks off the blade where the pulse is passing
    float bt = fract(t*KH_RATE)/KH_RATE, bn = floor(t*KH_RATE);
    for (int i = 0; i < 18; i++){
      float fi = float(i);
      float y0 = min(bt*KH_V, top) - 1.0 - 6.0*h11(fi*3.3 + bn);
      if (y0 < 0.5 || y0 > top) continue;
      vec3 dir = normalize(vec3(sin(6.2832*h11(fi*1.9 + bn)), 0.4*(h11(fi*4.4 + bn) - 0.3), cos(6.2832*h11(fi*1.9 + bn))));
      float age = bt - y0/KH_V;
      if (age < 0.0) continue;
      float v = (4.0 + 5.0*h11(fi*7.1 + bn))*pump;
      vec3 p0 = vec3(0.0, y0, 0.0) + dir*(0.4 + v*(1.0 - exp(-age*3.0))/3.0);
      vec3 p1 = p0 - dir*(0.3 + 0.6*exp(-age*3.0));
      float sk;
      float dk = raySeg(ro, rd, p1, p0, sk);
      fxl += mix(white, rose, smoothstep(0.0, 0.6, age))*(halo(dk, max(0.05, px))*1.6 + halo(dk, 0.3)*0.3)*exp(-age*2.5)*sk;
    }
    // ---- a clash: the heart skips, a white flare floods the blade and a strong ring bursts out
    if (cl > 0.01){
      float R = 1.0 + 6.5*(1.0 - cl);
      float wr = 0.3 + 0.5*(1.0 - cl);
      vec3 o = ro - vec3(0.0, L*0.5, 0.0); o.y *= 3.0;
      vec3 dd = rd; dd.y *= 3.0; dd = normalize(dd);
      float rr = length(o + dd*max(dot(-o, dd), 0.0));
      float sh = clamp(wr/sqrt(max(R*R - rr*rr, wr*wr)), 0.0, 1.0)*smoothstep(R + wr, R - 2.0*wr, rr) + halo(abs(rr - R), wr*1.4)*0.5;
      float tc;
      fxl += (pale*sh*1.2)*cl + white*halo(rayPtRaw(ro, rd, vec3(0.0, L*0.5, 0.0), tc), 1.5)*pow(cl, 2.0)*0.6;
    }
    light += fxl*fx;
  }
  return light;
}
// the blade: Sol's glow and core from the file, swelling and blazing as each pulse passes and
// dimming between beats
vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  float p = kh_pulse(b.y)*(1.0 + 0.6*clamp(uSwing, 0.0, 1.0)) + 1.2*clamp(uClash, 0.0, 1.0);
  return g*(0.6 + 0.9*p);
}
vec3 sig_core(vec3 c, vec2 b, float px){
  float p = kh_pulse(b.y) + 1.2*clamp(uClash, 0.0, 1.0);
  return c*(0.85 + 0.5*p);
}

// ---- round 14 (Creyon): arteries of lifeblood and sparks ----
// Three arteries of light winding slowly round the blade, wider than its veins so they stand
// clear of its glow (the point of artery fv at height y, thinning towards the tip).
vec3 kh_artery(float fv, float y, float top){
  float dir = fv < 0.5 ? 1.0 : (fv < 1.5 ? -1.0 : 1.0);
  float r = (1.7 + 0.35*sin(y*0.22 + fv*2.1))*(1.0 - 0.6*smoothstep(top - 6.0, top, y));
  return around(r, fv/3.0 + dir*y*0.022, y);
}
// Lifeblood: the arteries glow faintly rose, and bright drops of light surge up them without
// pause (a steady clock, so nothing races when turning or running), flaring white as each
// heartbeat passes them; sparks crackle off the arteries and fly out. All of it starts above
// the wearer's head (0.2 of the length) and fades out under the tip.
vec3 kh_lifeblood(vec3 ro, vec3 rd, float px){
  vec3 acc = vec3(0.0);
  float L = uLength, t = uTime;
  float top = L*clamp(uAge/0.45, 0.0, 1.0);
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float pump = 1.0 + 0.6*sw;
  float s;
  if (raySegRaw(ro, rd, vec3(0.0, 0.0, 0.0), vec3(0.0, top, 0.0), s) > 6.0) return acc;
  vec3 rose = vec3(1.0, 0.3, 0.5), hot = vec3(1.0, 0.75, 0.85), white = vec3(1.0);
  float y0 = L*0.2, y1 = max(top - 0.5, y0);
  for (int v = 0; v < 3; v++){
    float fv = float(v);
    // the artery: a thin rose line, brighter where the beat is passing
    vec3 prev = kh_artery(fv, y0 - 1.5, top);
    float best = 1e3, by = y0;
    for (int k = 1; k <= 18; k++){
      float yy = mix(y0 - 1.5, y1, float(k)/18.0);
      vec3 q = kh_artery(fv, yy, top);
      float sv;
      float dv = raySeg(ro, rd, prev, q, sv);
      if (dv < best){ best = dv; by = mix(prev.y, q.y, sv); }
      prev = q;
    }
    float pb = kh_pulse(by)*pump + cl;
    float fade = smoothstep(y0 - 1.5, y0 + 1.0, by)*(by < top ? 1.0 : 0.0);
    acc += (mix(rose, hot, min(pb, 1.0))*halo(best, max(px, 0.035))*(0.7 + 1.3*pb) + rose*halo(best, 0.35)*(0.2 + 0.45*pb))*fade;
    // the lifeblood: four drops on each artery, a bright head and a tapering tail
    for (int j = 0; j < 4; j++){
      float fj = float(j);
      float f = fract(t*0.3 + fj/4.0 + 0.29*fv);
      float y = mix(y0 - 3.0, L + 2.0, f);
      if (y > top + 1.0) continue;
      vec3 a = kh_artery(fv, y - 3.0, top);
      vec3 m = kh_artery(fv, y - 1.4, top);
      vec3 b = kh_artery(fv, y, top);
      float s1, s2;
      float d1 = raySeg(ro, rd, a, m, s1);
      float d2 = raySeg(ro, rd, m, b, s2);
      float d = min(d1, d2);
      float tail = d2 < d1 ? 0.45 + 0.55*s2 : 0.45*s1;
      float fd = smoothstep(y0 - 2.0, y0 + 0.5, y)*(1.0 - smoothstep(top - 1.5, top + 1.0, y));
      float pv = kh_pulse(y)*pump + cl;
      float head = halo(length(ro + rd*max(dot(b - ro, rd), 0.0) - b), 0.22);
      acc += (mix(hot, white, min(pv, 1.0))*halo(d, max(px, 0.06))*2.2 + rose*halo(d, 0.4)*0.6)*tail*tail*fd*(0.8 + 0.8*pv)
           + white*head*fd*(0.9 + 1.2*pv);
    }
  }
  // the crackle: sparks jumping off an artery and flying out, each on its own steady cycle
  for (int i = 0; i < 16; i++){
    float fi = float(i);
    float cyc = t*1.1*(0.7 + 0.6*h11(fi*2.3 + 5.0)) + h11(fi*6.1 + 1.0);
    float k = floor(cyc), q = fract(cyc);
    if (q > 0.4) continue;
    float age = q/0.4;
    float fv = mod(fi, 3.0);
    float y = mix(y0, y1, h11(fi*3.7 + k*1.31));
    vec3 p = kh_artery(fv, y, top);
    vec3 dir = normalize(normalize(vec3(p.x, 0.0, p.z) + vec3(1e-4, 0.0, 0.0)) + vec3(0.0, 0.8*(h11(fi*1.7 + k*2.9) - 0.25), 0.0));
    vec3 p1 = p + dir*(0.1 + 4.5*age*pump);
    vec3 p0 = p + dir*(0.1 + 4.5*max(age - 0.22, 0.0)*pump);
    float sk;
    float dk = raySeg(ro, rd, p0, p1, sk);
    float fd = smoothstep(y0 - 1.0, y0 + 1.0, y);
    acc += mix(white, rose, age)*(halo(dk, max(px, 0.045))*2.2 + halo(dk, 0.25)*0.4)*(1.0 - age)*(0.4 + 0.6*sk)*fd;
  }
  return acc;
}

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = (r8_vol(ro, rd, px) + kh_lifeblood(ro, rd, px)*uFx)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 rose = vec3(1.0, 0.3, 0.5);
  float ph = fract(t*KH_RATE);
  float beat = exp(-ph*9.0) + 0.6*exp(-max(ph - 0.22, 0.0)*9.0)*step(0.22, ph);
  fx += fxFlare(ro, rd, vec3(0.0, L*0.35, 0.0), px, 0.3, 4.0 + 2.0*beat, 4.0, 0.785, vec3(1.0, 0.85, 0.9))*(0.4 + 1.2*beat)*part(1.0);
  fx += fxAura(ro, rd, y0, y1, 2.6, rose)*(0.5 + 1.0*beat)*part(1.0);
  return col + fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 70 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.7; }
