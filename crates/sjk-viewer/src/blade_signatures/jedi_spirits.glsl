// Jedi Spirits: the light of the Jedi who became one with the Force. A soft blade breathing
// slowly; warm lights lift off it one by one and float away upward, fading, a pale spirit mist
// rises round it, three spirits climb round it in a slow spiral trailing long veils of light, an
// echo of the blade sways beside it and afterimages linger when it swings (round 15: mythical,
// its ribbons, shared with the Aurodium, became the climbing spirits). A clash sends a warm ring.
// @part Spirit lights
// @part Spirit mist
// @part Ascending spirits
// @room 8
// @palette {"corona.rim_cool":[0.55,0.4,0.15],"corona.rim_hot":[1.0,0.85,0.55],"corona.inner":[1.0,0.97,0.88],"core.white":[1,1,0.95],"core.fringe_cool":[1.0,0.8,0.5],"core.fringe_hot":[1.0,0.93,0.75],"light.color":[1.0,0.85,0.55],"trail":[1.0,0.85,0.55]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g*(0.9 + 0.15*sin(uTime*1.2));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 warm = vec3(1.0, 0.85, 0.55);
  for (int i = 0; i < 10; i++){
    float fi = float(i), ta;
    float f = fract(t*0.08 + h11(fi*3.7));
    vec3 p = around(0.8 + 3.5*f, h11(fi*5.3) + 0.1*sin(t*0.4 + fi), mix(y0, y1, h11(fi*2.1 + floor(t*0.08 + h11(fi*3.7)))) + 6.0*f);
    float d = rayPt(ro, rd, p, ta);
    fx += warm*(halo(d, 0.25)*1.2 + halo(d, 0.8)*0.3)*sin(3.1416*f)*part(0.0);
  }
  fx += fxSmoke(ro, rd, y0, y1, 3.8, 1.0, 4.9, vec3(0.75, 0.8, 0.9))*0.6*part(1.0);
  fx += fxMotes(ro, rd, px, y0, y1, 1.0, 4.0, 22.0, 0.03, 5.9, vec3(1.0, 0.9, 0.7))*0.9*part(0.0);
  // round 15b: a Force ghost's pale blue glow round it, breathing
  fx += fxAura(ro, rd, y0, y1, 2.8, vec3(0.55, 0.75, 1.0))*(0.35 + 0.15*sin(t*0.9))*part(1.0);
  return fx*uFx;
}

// round 15: three spirits climbing round the blade in a slow spiral, each a soft light trailing a
// long fading veil; they circle the blade, so they are drawn in 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  // round 17 (Creyon: the spirits stuttered at the tip): they rise to below the tip and fade
  // there, and a spirit's trail is only its own climb (it no longer reached back into the end of
  // the climb before, at the top, while the spirit began again at the bottom)
  float y0 = L*0.12, y1 = L*0.9;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) > 5.5) return fx;
  for (int j = 0; j < 3; j++){
    float fj = float(j);
    float per = 9.0 + 1.5*fj;
    float head = t/per + fj/3.0;
    float k = floor(head);
    float best = 1e3, bs = 0.0;
    vec3 prev = vec3(0.0);
    bool have = false;
    for (int i = 0; i <= 14; i++){
      float tt = t - float(i)*0.12;
      float c = tt/per + fj/3.0;
      if (floor(c) < k) break;
      float q = fract(c);
      vec3 p = around(2.1 + 0.9*q + 0.25*sin(tt*0.8 + fj), tt*0.11 + fj/3.0, mix(y0, y1, q));
      if (have){
        float d = raySeg(ro, rd, prev, p, s);
        if (d < best){ best = d; bs = (float(i) - 1.0 + s)/14.0; }
      }
      prev = p;
      have = true;
    }
    float q0 = fract(head);
    float fade = smoothstep(0.0, 0.12, q0)*(1.0 - smoothstep(0.7, 0.97, q0));
    float veil = 1.0 - bs;
    vec3 warm = vec3(1.0, 0.88, 0.62), pale = vec3(0.75, 0.86, 1.0);
    fx += (vec3(1.0)*halo(best, max(0.07, px))*pow(veil, 5.0)*3.0 + mix(pale, warm, veil)*halo(best, 0.45 + 0.45*bs)*veil*1.4
           + pale*halo(best, 1.1)*0.1*veil)*fade;
  }
  return fx*uFx;
}
