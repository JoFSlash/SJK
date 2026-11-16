// Hearth: a home fire, the second Unique blade (Sol, 25/10/2026), a thank-you given by hand. A
// warm amber blade of glowing coals that breathe, embers rising off it, kindling from the hilt
// as it lights and cooling to a deep red as it goes out. Tongues of flame lick up its edges, the
// air above it shimmers with heat, and the fire is a blaze, taller, brighter and crackling,
// sparks popping off it; a ring of hearthstones stands round its emitter in the firelight, ash
// drifts off it, and its swings shed ash.
// @part Hearth flames
// @part Blaze
// @part Hearthstones
// @embers Embers
// @haze Heat shimmer
// @petals {"part":2,"kind":"ash","per_unit":0.04,"life":3.2,"size":0.9,"fall":10.0,"drift":8.0}
// @room 9
// @solid 1
// @palette {"corona.rim_cool":[0.7,0.2,0.04],"corona.rim_hot":[1.0,0.56,0.18],"corona.inner":[1.0,0.8,0.48],"core.white":[1.0,0.95,0.84],"core.fringe_cool":[1.0,0.5,0.16],"core.fringe_hot":[1.0,0.78,0.42],"light.color":[1.0,0.58,0.26],"trail":[1.0,0.58,0.24]}

const vec3 HE_RED = vec3(0.62, 0.1, 0.02);
const vec3 HE_AMBER = vec3(1.0, 0.5, 0.12);
const vec3 HE_GOLD = vec3(1.0, 0.82, 0.42);

// the fire's colour at heat h (0 a dull red, 1 white-gold)
vec3 he_fire(float h){
  vec3 c = mix(HE_RED, HE_AMBER, smoothstep(0.0, 0.5, h));
  return mix(c, vec3(1.0, 0.93, 0.7), smoothstep(0.6, 1.0, h));
}

// how lit the fire is: kindling up from the hilt as the blade lights, cooling as it goes out
float he_kindle(float y){
  if (uAge >= 2000.0) return 1.0 - 0.65*smoothstep(2000.0, 2000.7, uAge);
  return smoothstep(-5.0, 2.0, uAge*45.0 - y);
}

// tongues of flame licking up both edges of the blade, `reach` units out at most, in the flat
// picture round it (as Sol's flames are): each edge a sheet of fire whose tongues rise, sway and
// break off; returns the light
vec3 he_flames(vec2 b, float r, float reach, float speed, float hot, float seed){
  float L = uLength;
  float d = abs(b.x) - 0.35*r;
  if (d > reach || b.y < 0.5 || b.y > L + 1.5) return vec3(0.0);
  float side = (b.x < 0.0 ? 17.0 : 0.0) + seed;
  float rise = b.y*0.2 - uTime*speed*0.5;
  float n = fbm3(vec2(abs(b.x)*0.9 + side, rise)) + 0.3*vn(vec2(b.y*0.7 + side, uTime*2.1));
  // a tongue where the noise is high, gaps between them, the fire thinning toward the tip
  float tall = reach*1.7*max(n - 0.42, 0.0)*(1.0 - smoothstep(L*0.85, L + 1.0, b.y));
  float f = smoothstep(0.0, 0.45, tall - d)*smoothstep(-0.45, 0.0, d)*smoothstep(0.0, 0.25, tall);
  if (f <= 0.0) return vec3(0.0);
  // each tongue fading toward its tip
  float up = clamp(d/max(tall, 0.05), 0.0, 1.0);
  f *= 1.0 - up*up;
  // hot at its root on the blade, red at its tip
  float heat = clamp(1.0 - d/max(tall, 0.05), 0.0, 1.0)*(0.6 + 0.4*hot);
  return he_fire(heat)*f*(0.55 + 0.3*hot);
}

// the blade: Sol's glow from the file as a bed of glowing coals, their hearts bright and the
// cracks between them dark red, each breathing on its own; and the flames along its edges
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  vec2 cell;
  float e = voro(vec2(b.x*2.4/max(r, 0.3), b.y*0.55), cell);
  float breathe = 0.65 + 0.35*sin(uTime*(0.6 + 0.5*h21(cell)) + 6.2832*h21(cell + 3.1));
  // the coals along the blade's body only, not round the emitter or out in its glow
  float body = smoothstep(0.4, 1.6, b.y)*(1.0 - smoothstep(0.35*r, 0.75*r, abs(b.x)));
  float coal = mix(0.75, smoothstep(0.02, 0.22, e)*breathe, body);
  float k = he_kindle(b.y);
  vec3 col = l*mix(HE_RED*0.9, mix(HE_AMBER, HE_GOLD, coal*coal*0.55), coal)*1.25;
  col = mix(l*HE_RED*0.55, col, k);
  // a slow flicker over the whole bed, as a fire's light wavers
  col *= 0.9 + 0.1*sin(uTime*7.3 + b.y*0.3) + 0.06*sin(uTime*13.1);
  float sw = clamp(uSwing, 0.0, 1.0);
  col += he_flames(b, r, 1.8 + 0.5*sw, 1.5, 0.0, 0.0)*part(0.0)*k*uFx;
  // the blaze: the flames taller and hotter, burning faster
  col += he_flames(b, r, 3.2 + 0.7*sw, 2.2, 1.0, 41.0)*0.8*part(1.0)*k*uFx;
  return col;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*vec3(1.0, 0.88, 0.7)*(0.55 + 0.45*he_kindle(b.y)); }

// the blaze's crackle: now and then a coal pops, a flash and a few sparks flying off in arcs
vec3 he_pops(vec3 ro, vec3 rd, float px){
  float L = uLength;
  vec3 acc = vec3(0.0);
  float s;
  for (int i = 0; i < 4; i++){
    float fi = float(i);
    float rate = 0.9 + 0.5*h11(fi*3.7);
    float k = floor(uTime*rate + h11(fi*5.1));
    float f = fract(uTime*rate + h11(fi*5.1));
    if (h11(k*1.3 + fi*7.7) < 0.45) continue;
    float a = h11(k*2.9 + fi);
    vec3 o = around(0.35, a, L*(0.15 + 0.75*h11(k*4.1 + fi*2.3)));
    acc += vec3(1.0, 0.85, 0.5)*halo(rayPtRaw(ro, rd, o, s), 0.5)*exp(-f*9.0)*1.2;
    for (int j = 0; j < 4; j++){
      float fj = float(j);
      float aj = a + (h11(k*6.1 + fj*3.3) - 0.5)*0.35;
      float fly = 1.2 + 2.0*h11(k*8.3 + fj);
      vec3 p0 = o + around(fly*f, aj, 1.6*f - 2.2*f*f);
      float fb = max(f - 0.06, 0.0);
      vec3 p1 = o + around(fly*fb, aj, 1.6*fb - 2.2*fb*fb);
      float d = raySeg(ro, rd, p1, p0, s);
      acc += he_fire(1.0 - f)*halo(d, max(0.05, px))*1.6*(1.0 - smoothstep(0.5, 1.0, f));
    }
  }
  return acc;
}

// the blaze's popping coals, and ash drifting down off the blade
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength;
  vec3 fx = he_pops(ro, rd, px)*part(1.0);
  fx += fxFlakes(ro, rd, px, L*0.3, L*0.95, 3.0, 10.0, 1.6, 0.14, 0.6, 1.5, 0.6, 4.1, vec3(0.5, 0.45, 0.42))*0.35*part(2.0);
  return fx*uFx;
}

// a ring of hearthstones round the emitter: seven rounded river stones, grey-brown, lit warm
// from the fire on the side toward the blade, the far side in shadow, in three dimensions
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 res = vec4(0.0);
  if (fxNear(ro, rd, -1.0, 3.5) > 4.5) return res;
  float t;
  for (int i = 0; i < 7; i++){
    float fi = float(i);
    float a = fi/7.0 + 0.04*h11(fi*3.1);
    float R = 1.9 + 0.15*h11(fi*5.3);
    vec3 c = around(R, a, 1.2 + 0.2*h11(fi*2.7));
    float size = 0.72 + 0.15*h11(fi*7.9);
    float d = rayPt(ro, rd, c, t);
    if (d > size*1.4) continue;
    // a flattened round stone facing the camera, shaded as a ball lit from the blade
    vec3 q3 = ro + rd*t - c;
    vec3 fu = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
    vec3 fw = cross(fu, rd);
    vec2 q = vec2(dot(q3, fu), dot(q3, fw))/size;
    q.y *= 1.35;
    float rr = length(q);
    float cov = fillD(rr - 1.0, px/size);
    if (cov <= 0.0) continue;
    vec3 n = normalize(fu*q.x + fw*q.y - rd*sqrt(max(1.0 - rr*rr, 0.0)));
    vec3 toward = normalize(vec3(-c.x, 0.4, -c.z));
    float lit = max(dot(n, toward), 0.0);
    vec3 stone = mix(vec3(0.24, 0.21, 0.19), vec3(0.42, 0.37, 0.33), h11(fi*4.4));
    stone *= 0.9 + 0.2*vn(q*3.0 + fi*5.0);
    vec3 col = stone*0.7 + HE_AMBER*(0.2 + 0.75*lit)*(0.85 + 0.15*sin(uTime*7.3 + fi));
    res = vec4(mix(res.rgb, col, cov), max(res.a, cov));
  }
  return res*part(2.0)*uFx;
}

// the hearthstones' warm glow where the fire lights them
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  float t;
  float d = rayPtRaw(ro, rd, vec3(0.0, 1.2, 0.0), t);
  return HE_AMBER*halo(d, 2.4)*0.18*part(2.0)*uFx;
}
