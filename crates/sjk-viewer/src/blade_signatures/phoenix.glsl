// Phoenix Squadron: the fire of the Rebellion's Phoenix as a blade. An orange-gold blade of living
// flame that rises up it in waves; a plume of flames rises up it from hilt to tip, embers
// cascade upward off it and burning sparks stream away behind; every few seconds the fire is
// reborn: it dims to glowing embers and then flares up from the hilt in a rush of flame. A swing
// feeds the plume; a clash bursts a ring of fire.
// @part Phoenix fire
// @part Sparks
// @part Rebirth flare
// @room 10
// @palette {"corona.rim_cool":[0.60,0.15,0.02],"corona.rim_hot":[1.0,0.55,0.10],"corona.inner":[1.0,0.88,0.55],"core.white":[1.0,0.97,0.88],"core.fringe_cool":[1.0,0.45,0.08],"core.fringe_hot":[1.0,0.80,0.40],"light.color":[1.0,0.55,0.15],"trail":[1.0,0.55,0.15]}

const float PQ_P = 9.0;
float pq_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
// rebirth: 1 while the fire burns, low while it is embers, a front racing up as it flares
float pq_life(float y){
  float ph = mod(uTime, PQ_P);
  float dim = smoothstep(6.5, 7.2, ph)*(1.0 - smoothstep(7.6, 7.7, ph));
  float front = (ph - 7.6)*40.0;
  float reborn = ph > 7.6 ? smoothstep(front + 1.0, front - 1.0, y) : 1.0;
  return mix(1.0, 0.25, dim)*mix(0.25, 1.0, reborn) + (ph > 7.6 ? halo(y - front, 2.0)*1.5 : 0.0);
}
vec3 pq_fire(float h){
  vec3 c = mix(vec3(0.6, 0.08, 0.0), vec3(1.0, 0.45, 0.06), smoothstep(0.0, 0.45, h));
  return mix(c, vec3(1.0, 0.9, 0.6), smoothstep(0.55, 1.0, h));
}
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float t = uTime, top = pq_top(), s, ta;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 col = vec3(0.0);
  if (top < 0.5) return col;
  // the plume: flames rising up the blade from hilt to tip (round 13, Creyon: no loops round
  // the blade; it was two strands spiralling round it)
  col += fxSmoke(ro, rd, 0.0, top, 1.8 + 0.8*sw, 7.0, 3.1, vec3(1.0, 0.42, 0.06))*1.8*pq_life(top*0.5);
  col += fxSmoke(ro, rd, 0.0, top, 1.0, 9.0, 6.4, vec3(1.0, 0.8, 0.35))*1.2*pq_life(top*0.5);
  for (int k = 0; k < 0; k++){
    float fk = float(k);
    vec3 prev = around(0.5, fk*0.5 - t*0.6, 0.0);
    for (int j = 1; j <= 14; j++){
      float u = float(j)/14.0;
      float y = u*top;
      float R = 0.7 + 1.4*u + 0.4*sin(u*10.0 - t*4.0 + fk*3.0) + 0.8*sw;
      vec3 q = around(R, fk*0.5 + u*1.6 - t*0.6, y);
      float d = raySeg(ro, rd, prev, q, s);
      float uu = (float(j) - 1.0 + s)/14.0;
      float fl = 0.6 + 0.4*vn(vec2(uu*12.0 - t*6.0, fk*5.0));
      float w = 0.18 + 0.25*fl;
      col += (pq_fire(fl)*halo(d, max(w, px))*1.4 + vec3(1.0, 0.35, 0.03)*halo(d, w*3.0)*0.3)*pq_life(y)*(0.7 + 0.5*sw);
      prev = q;
    }
  }
  // embers cascading upward off it
  for (int i = 0; i < 26; i++){
    float fi = float(i);
    float f = fract(t*(0.35 + 0.25*h11(fi*2.1)) + h11(fi*5.7));
    float k = floor(t*(0.35 + 0.25*h11(fi*2.1)) + h11(fi*5.7));
    float y0 = (0.03 + 0.8*h11(fi*3.3 + k))*top;
    vec3 p = around(0.5 + 2.8*f, h11(fi*7.9 + k) + 0.3*f, y0 + 5.0*f);
    vec3 q = around(0.5 + 2.8*max(f - 0.05, 0.0), h11(fi*7.9 + k) + 0.3*max(f - 0.05, 0.0), y0 + 5.0*max(f - 0.05, 0.0));
    float d = raySeg(ro, rd, q, p, s);
    col += pq_fire(1.0 - f)*(halo(d, max(0.05, px))*1.8 + halo(d, 0.25)*0.2)*sin(3.1416*f)*pq_life(y0);
  }
  // a clash: a ring of fire bursting out
  if (cl > 0.01){
    float yc = uLength*0.6;
    vec3 h = rayAt(ro, rd, yc);
    if (h.z > 0.0){
      float rr = hideD(abs(length(h.xy) - 0.6 - 5.5*(1.0 - cl)), behindBlade(ro, rd, h.z));
      col += pq_fire(cl)*(halo(rr, 0.3 + 0.5*(1.0 - cl))*1.8)*cl;
    }
    float dc = rayPtRaw(ro, rd, vec3(0.0, yc, 0.0), ta);
    col += vec3(1.0, 0.9, 0.6)*halo(dc, 1.6)*cl*cl*1.6;
  }
  return col*uFx;
}
// the blade: Sol's glow and core from the file, living flame rising up it in waves, dimming to
// embers and flaring up again as it is reborn
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float wave = 0.75 + 0.25*sin(b.y*0.7 - uTime*6.0 + 2.0*vn(vec2(b.x*2.0, b.y*0.4 - uTime*2.0)));
  return g*wave*pq_life(b.y)*(1.0 + 0.6*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(0.4 + 0.6*min(pq_life(b.y), 1.2)); }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 gold = vec3(1.0, 0.8, 0.35);
  // round 13: every few seconds the phoenix is reborn in a flare of fire (no ring round it)
  float rb = fract(t/6.0);
  fx += fxFlare(ro, rd, vec3(0.0, L*0.5, 0.0), px, 0.4, 3.0 + 6.0*rb, 6.0, rb*0.6, gold)*exp(-rb*4.0)*1.4*part(2.0);
  fx += fxSparks(ro, rd, px, y0, y1, 20.0, 6.5, 1.0, 5.6, gold)*part(1.0);
  return col + fx*uFx;
}
