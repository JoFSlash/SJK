// Event Horizon: the blade is a black hole stretched into a line. Its heart is a rod of pure
// darkness outlined by a razor-thin photon ring, with light running down its edges as it is
// pulled in. Two real accretion disks of hot swirling matter circle the blade at two heights,
// turning, brighter on the side rushing toward you, each wrapped in a thin lensed photon ring;
// streams of matter spiral in along the blade from the disks in pulses of light. Seen from above
// the disks are full whirlpools inside a bright Einstein ring; from the side, thin blazing lines.
// A swing spins the disks up and speeds the infalling streams; a clash fires a gravity wave.
// @part Accretion disks
// @part Infalling streams
// @motes Infalling stars
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.30,0.06,0.70],"corona.rim_hot":[0.70,0.40,1.0],"corona.inner":[0.9,0.85,1.0],"core.white":[0.95,0.9,1.0],"core.fringe_cool":[0.40,0.12,0.90],"core.fringe_hot":[0.80,0.55,1.0],"light.color":[0.55,0.25,1.0],"trail":[0.60,0.35,1.0]}

float eh_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
float eh_h(float k){ return uLength*(0.36 + 0.34*k) + 0.9*sin(uTime*(0.4 - 0.07*k) + k*2.0); }
// how far a disk has grown (it spreads out from the blade as the ignition passes its height)
float eh_grow(float h){ return smoothstep(h - 1.0, h + 5.0, eh_top()); }

// one accretion disk at height h: clumps of hot matter orbiting the blade on many rings,
// brighter on the side rushing toward you, a thin blazing ring at its inner edge; each clump
// a piece facing the camera, as Sol's Void blade orbits its black holes
vec3 eh_disk(vec3 ro, vec3 rd, float h, float k, float tmax, float g, float px){
  vec3 vio = vec3(0.62, 0.36, 1.0), white = vec3(1.0, 0.96, 1.0);
  vec3 acc = vec3(0.0);
  float tq;
  if (g < 0.01 || rayPtRaw(ro, rd, vec3(0.0, h, 0.0), tq) > 7.5*g) return acc;
  float sw = clamp(uSwing, 0.0, 1.0);
  float boost = 1.0 + 0.6*sw + 1.2*clamp(uClash, 0.0, 1.0);
  for (int i = 0; i < 40; i++){
    float fi = float(i);
    float r = (0.95 + 4.6*pow(h11(fi*3.1 + k*7.0), 1.6))*g;
    float a = h11(fi*5.7 + k) + uTime*(0.55)/(0.4 + 0.35*r);
    vec3 p = around(r, a, h + 0.08*(h11(fi*2.2) - 0.5)*r);
    // each clump a short streak along its orbit
    vec3 q = around(r, a - 0.05/(0.5 + 0.3*r), p.y);
    float s;
    float d = raySeg(ro, rd, q, p, s);
    float T = clamp(1.25*g/r, 0.0, 1.0);
    vec3 vt = normalize(vec3(cos(6.2832*a), 0.0, -sin(6.2832*a)));
    float dop = dot(vt, -rd);
    float beam = pow(1.0 + 0.6*dop, 3.0);
    vec3 c = mix(vio, white, T*T*(0.6 + 0.4*clamp(dop, 0.0, 1.0)));
    acc += c*(halo(d, max(0.06 + 0.04*r, px))*1.1 + halo(d, 0.35 + 0.1*r)*0.25)*beam*(0.5 + 1.5*T)*boost*step(dot(p - ro, rd), tmax);
  }
  // the innermost stable orbit: a thin ring blazing round the hole
  vec3 hp = rayAt(ro, rd, h);
  if (hp.z > 0.0 && hp.z < tmax){
    float rr = hideD(abs(length(hp.xy) - 0.85*g), behindBlade(ro, rd, hp.z));
    acc += white*(halo(rr, max(0.04, px))*1.3 + vio*halo(rr, 0.3)*0.4)*boost;
  }
  return acc;
}

// all the effects round the blade, counted only up to tmax along the ray
vec3 eh_fx(vec3 ro, vec3 rd, float px, float tmax){
  float L = uLength, top = eh_top();
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 vio = vec3(0.62, 0.36, 1.0), white = vec3(1.0, 0.96, 1.0);
  vec3 light = vec3(0.0);
  float s, tq;
  for (int k = 0; k < 2; k++){
    float fk = float(k);
    float h = eh_h(fk), g = eh_grow(h);
    if (g < 0.01) continue;
    light += eh_disk(ro, rd, h, fk, tmax, g, px);
    // the lensed photon ring: a thin ring of bent light hugging the hole at the disk's height,
    // always facing you; brighter when seen from above
    float dc = rayPtRaw(ro, rd, vec3(0.0, h, 0.0), tq);
    if (tq < tmax){
      vec3 q = ro + rd*tq - vec3(0.0, h, 0.0);
      float side = dot(normalize(vec3(-q.z, 0.0, q.x) + 1e-4), -rd);
      float flick = 0.75 + 0.25*sin(atan(q.y, length(q.xz)*sign(q.x + 1e-4))*5.0 - uTime*6.0);
      float rr = abs(dc - 0.95*g);
      float up = 0.55 + 0.9*rd.y*rd.y;
      light += (white*halo(rr, max(0.035, px))*1.8 + vio*halo(rr, 0.25)*0.35)*(0.7 + 0.5*side)*flick*up*g;
    }
    // matter streams spiralling in along the blade from the disk's inner edge, up and down
    float da = raySegRaw(ro, rd, vec3(0.0, h - 7.5, 0.0), vec3(0.0, h + 7.5, 0.0), s);
    if (da > 2.6) continue;
    for (int j = 0; j < 4; j++){
      float fj = float(j);
      float dir = j < 2 ? 1.0 : -1.0;
      float a0 = fj*0.5 + fk*0.3 - uTime*(0.35);
      vec3 prev = around(1.7*g, a0, h);
      for (int i = 1; i <= 8; i++){
        float u = float(i)/8.0;
        vec3 p = around(mix(1.7, 0.36, sqrt(u))*g, a0 + u*1.6*dir, h + dir*u*7.0*g);
        float d = raySeg(ro, rd, prev, p, s);
        if (d < 0.6){
          float uu = u - (1.0 - s)/8.0;
          vec3 qp = mix(prev, p, s);
          float tt = dot(qp - ro, rd);
          float pulse = 0.35 + 0.65*pow(0.5 + 0.5*sin(uu*14.0 - uTime*(7.0) + fj*1.3), 3.0);
          float fade = (1.0 - smoothstep(0.6, 1.0, uu))*smoothstep(0.0, 0.08, uu);
          vec3 c = mix(vio, white, uu*0.7 + 0.2);
          light += c*(halo(d, max(0.035, px))*1.3 + halo(d, 0.18)*0.35)*pulse*fade*step(tt, tmax);
        }
        prev = p;
      }
    }
  }
  // the Einstein ring: from above (or below) a bright ring of lensed light circles the blade
  float aligned = smoothstep(0.8, 0.97, -rd.y);
  if (aligned > 0.0){
    float hE = eh_h(1.0);
    float dA = rayPtRaw(ro, rd, vec3(0.0, hE, 0.0), tq);
    float er = abs(dA - 3.1 - 0.15*sin(uTime*1.3));
    vec3 pa = ro + rd*tq;
    float ang = atan(pa.z, pa.x);
    float sh = 0.6 + 0.4*sin(ang*3.0 + uTime*2.0);
    light += (white*halo(er, max(0.05, px))*2.2 + vio*halo(er, 0.5)*0.45)*sh*aligned*eh_grow(hE);
  }
  // a clash: a gravity wave (a sphere of bent light) bursts out from the struck point
  if (cl > 0.01){
    float dc = rayPtRaw(ro, rd, vec3(0.0, L*0.6, 0.0), tq);
    float rr = abs(dc - 0.6 - 5.5*(1.0 - cl));
    light += (white*halo(rr, 0.1)*2.2 + vio*halo(rr, 0.8)*0.5)*cl*step(tq, tmax);
  }
  return light;
}

vec3 r8_vol(vec3 ro, vec3 rd, float px){
  // The blade is Sol's glow and core from the file round the black rod (sig_glow,
  // sig_solid); here only the effects, each a piece placed round it in 3D.
  return eh_fx(ro, rd, px, 1e9)*uFx;
}
// the black rod, drawn over Sol's glow as his solid blades are: pure darkness from just
// above the emitter to the tip
vec4 sig_solid(vec4 o, vec2 b, float px){
  float top = eh_top();
  if (top < 1.3) return o;
  float a = fillD(abs(b.x) - 0.28, px)*smoothstep(1.0, 1.6, b.y)*(1.0 - smoothstep(top - 0.6, top - 0.2, b.y));
  return vec4(mix(o.rgb, vec3(0.01, 0.0, 0.02), a*0.96), max(o.a, a*0.96));
}
// the blade: Sol's glow from the file, light streaming down its edges as it falls in, a razor
// photon ring at the rod's rim
vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  float stream = 0.65 + 0.35*sin(b.y*0.9 + uTime*12.0) + 0.3*pow(0.5 + 0.5*sin(b.y*0.23 + uTime*5.0), 6.0);
  float rim = exp(-pow((abs(b.x) - 0.33)/max(0.06, px), 2.0))*smoothstep(1.0, 1.6, b.y)*(1.0 - smoothstep(eh_top() - 0.6, eh_top() - 0.2, b.y));
  return g + vec3(1.0, 0.96, 1.0)*rim*(1.2*stream + 1.5*clamp(uClash, 0.0, 1.0));
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
  vec3 vio = vec3(0.6, 0.3, 1.0);
  fx += fxCoils(ro, rd, px, L*0.04, y1, 2.6, 1.6, -0.6, 3.0, vio)*part(1.0);
  for (int i = 0; i < 3; i++){
    float q = fract(t*0.4 + float(i)/3.0);
    fx += fxShock(ro, rd, px, mix(L*0.3, L*0.8, float(i)*0.5), q, 8.0, vio)*0.45*part(1.0);
  }
  return col + fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 75 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.75; }
