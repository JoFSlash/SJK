// Mortis: the realm of the Force wielders, the Father, the Daughter and the Son, as a blade
// (Creyon's round 9, rebuilt as Sol builds his). The blade is Sol's glow and core from the file,
// and over it Mortis turns from day to night and back: by day it glows pale and serene, by night
// a deep crimson, turning every second time the ghost blades fold back into it. Round it in 3D stand two ghost blades at other angles, the Daughter's light and
// the Son's dark, slowly turning round it like the two sides held in check by the Father. By day
// motes of the Daughter's light rise round it and soft beams shine out; by night the Son's red
// lightning crackles off it and embers sink. A swing pulls the ghost blades in to the blade; a
// clash flashes all three and throws a ring of light.
// @part Ghost blades
// @part The Daughter's light
// @part The Son's lightning
// @part The Father's ring
// @room 10
// @palette {"corona.rim_cool":[0.4,0.35,1.0],"corona.rim_hot":[0.7,0.5,1.0],"corona.inner":[0.5,0.8,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.5,0.6,1.0],"core.fringe_hot":[0.8,0.9,1.0],"light.color":[0.45,0.75,1.0],"trail":[0.5,0.7,1.0]}

const vec3 MO_DAY = vec3(0.6, 1.0, 0.8);
const vec3 MO_NIGHT = vec3(1.0, 0.1, 0.08);
// Round 16 (Creyon: it must change on a pattern that fits): the ghost blades swing out and fold
// back into the blade once every MO_P seconds; every second time they meet in it, the realm
// turns, day to night or night to day, with a flash where they meet.
const float MO_P = 8.0;
// how far the ghost blades lean out: 0 while they are folded into the blade
float mo_open(){ float q = fract(uTime/MO_P); return smoothstep(0.08, 0.32, q)*(1.0 - smoothstep(0.72, 0.97, q)); }
// day (1) or night (0) as seen `lag` cycles late (the change sweeping up the blade)
float mo_day_at(float lag){
  float c = uTime/MO_P - lag;
  float k = floor(c), q = fract(c);
  float now = mod(floor(k/2.0), 2.0) < 0.5 ? 1.0 : 0.0;
  float before = mod(floor((k - 1.0)/2.0), 2.0) < 0.5 ? 1.0 : 0.0;
  return mix(before, now, smoothstep(0.0, 0.18, q));
}
float mo_day(){ return mo_day_at(0.0); }
// the flash where they meet when the realm turns
float mo_turn(){
  float c = uTime/MO_P, k = floor(c), q = fract(c);
  return mod(k, 2.0) < 0.5 ? exp(-q*14.0) : exp(-q*30.0)*0.3;
}

vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  // round 19 (Creyon: the lower part of the blade had none of its effects, by day or night):
  // everything reaches down to just above the hilt
  float y0 = L*0.04, y1 = L*0.97;
  vec3 col = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) > 10.0) return col;
  float day = mo_day(), s;
  // ---- the ghost blades: the Daughter's and the Son's, tilted, turning round the blade ----
  for (int j = 0; j < 2; j++){
    float fj = float(j);
    float a = t*0.04 + fj*0.5;
    float tilt = (0.32 - 0.24*sw)*(1.0 - cl)*mo_open();
    vec3 pivot = vec3(0.0, L*0.06, 0.0);
    vec3 dir = normalize(vec3(sin(6.2832*a)*tilt*0.75, 1.0, cos(6.2832*a)*tilt*0.75));
    vec3 a0 = pivot + dir*1.2, a1 = pivot + dir*(L*0.86);
    float d = raySeg(ro, rd, a0, a1, s);
    float ends = smoothstep(0.0, 0.15, s)*(1.0 - smoothstep(0.85, 1.0, s));
    vec3 gc = j == 0 ? MO_DAY : MO_NIGHT;
    float lit = (j == 0 ? 0.35 + 0.65*day : 0.35 + 0.65*(1.0 - day))*(0.4 + 0.6*mo_open());
    col += (vec3(1.0)*halo(d, max(0.08, px))*0.7 + gc*halo(d, 0.4)*0.6 + gc*halo(d, 1.4)*0.12)*ends*lit*(0.8 + 1.5*cl)*part(0.0);
  }
  // ---- where they meet as the realm turns: a flash of both lights ----
  float turn = mo_turn();
  if (turn > 0.01){
    vec3 pv = vec3(0.0, L*0.6, 0.0);
    col += fxFlare(ro, rd, pv, px, 0.35, 3.0 + 5.0*turn, 4.0, t*0.3, mix(MO_NIGHT, MO_DAY, mo_day()))*turn*part(0.0);
  }
  // ---- by day: the Daughter's light rising, soft beams ----
  col += (fxMotes(ro, rd, px, y0, y1, 1.0, 6.0, 40.0, 0.06, 5.5, MO_DAY) + fxAura(ro, rd, y0, y1, 3.0, MO_DAY)*0.6
    + fxRays(ro, rd, px, y0, y1, 6.0, 4.5, 0.5, 3.3, MO_DAY)*0.45)*day*part(1.0);
  // ---- by night: the Son's lightning and sinking embers ----
  col += (fxArcs(ro, rd, px, y0, y1, 7.0, 5.0, 1.8, 2.6, MO_NIGHT) + fxMotes(ro, rd, px, y0, y1, 1.0, 5.0, 26.0, -0.07, 8.8, vec3(1.0, 0.3, 0.15))
    + fxSmoke(ro, rd, y0, y1, 2.2, -1.2, 4.4, vec3(0.5, 0.04, 0.03))*0.5)*(1.0 - day)*part(2.0);
  // the Father's balance: slow rings turning round the blade, half day, half night (round 19:
  // a second low on the blade, as the first stands high)
  for (int k = 0; k < 2; k++){
    float fk = float(k);
    vec3 h = rayAt(ro, rd, L*(fk < 0.5 ? 0.78 : 0.24) + sin(t*0.4 + fk*2.0));
    if (h.z > 0.0){
      float R = fk < 0.5 ? 3.2 : 2.6;
      float d = hideD(abs(length(h.xy) - R), behindBlade(ro, rd, h.z));
      float side = 0.5 + 0.5*sin(atan(h.x, h.y) - t*0.2*(fk < 0.5 ? 1.0 : -1.0) + fk*3.1);
      col += (vec3(1.0)*halo(d, max(0.05, px))*0.8 + mix(MO_NIGHT, MO_DAY, side)*halo(d, 0.3)*0.5)*part(3.0);
    }
  }
  col += fxAura(ro, rd, y0, y1, 3.0, mix(MO_NIGHT, MO_DAY, day));
  return col*uFx;
}

// the blade: Sol's glow and core from the file, pale and serene by day, crimson by night, the
// change sweeping up the blade
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = max(g.r, max(g.g, g.b));
  float day = mo_day_at(b.y*0.004);
  vec3 c = mix(MO_NIGHT*1.2, mix(MO_DAY, vec3(1.0), 0.25), day);
  return l*c*(1.0 + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
