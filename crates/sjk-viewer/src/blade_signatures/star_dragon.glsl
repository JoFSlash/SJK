// Ashla Radiance: the Ashla, the light side of the Force in the old Jedi teachings, held as a
// blade. A white-gold blade glowing calm and steady, slow waves of light rising through it; soft
// radiant light rises off it like a quiet flame, soft rays of light reach slowly out of it,
// motes of light drift upward round it and a faint echo of the blade sways beside it. Nothing is
// drawn on its tip. Round 15b (Creyon: mythicals must not look cheap): two great wings of light
// open from its middle, feathered with soft rays sweeping out and down like an angel's, breathing
// slowly. A swing brightens the light;
// a clash flares the blade. Round 16: four orbs of the Ashla's light circle it slowly in 3D, each
// trailing soft light, and its motes fade before the tip instead of being cut there.
// @part Radiant light
// @part Rays of light
// @part Wings of light
// @motes Light of the Ashla
// @room 10
// @palette {"corona.rim_cool":[0.55,0.42,0.15],"corona.rim_hot":[1.0,0.86,0.5],"corona.inner":[1.0,0.96,0.82],"core.white":[1.0,0.99,0.94],"core.fringe_cool":[1.0,0.82,0.45],"core.fringe_hot":[1.0,0.95,0.78],"light.color":[1.0,0.86,0.5],"trail":[1.0,0.86,0.55]}

const vec3 AR_GOLD = vec3(1.0, 0.84, 0.5);
const vec3 AR_PALE = vec3(1.0, 0.96, 0.85);

// two wings of light opening from the blade's middle, each a fan of soft feathered rays curving
// up and out, breathing slowly (flat as Sol's are)
vec3 ar_wings(vec3 ro, vec3 rd, float px){
  vec3 acc = vec3(0.0);
  float L = uLength, s;
  if (fxNear(ro, rd, L*0.3, L*0.9) > 7.0) return acc;
  float breathe = 0.85 + 0.15*sin(uTime*0.8);
  for (int m = 0; m < 2; m++){
    float side = m == 0 ? -1.0 : 1.0;
    for (int i = 0; i < 7; i++){
      float fi = float(i);
      float a = -0.25 + 0.22*fi;
      float len = (8.0 - 0.6*fi)*breathe;
      // round 19: the wings swept back from the spine in a shallow V, in three dimensions
      vec3 o = turnY(vec3(side*0.3, L*0.42 + 0.9*fi, 0.0), side*0.07);
      vec3 prev = o;
      float best = 1e3, bs = 0.0;
      for (int j = 1; j <= 6; j++){
        float u = float(j)/6.0;
        float ang = a + 0.35*u*u;
        vec3 q = o + turnY(vec3(side*cos(ang), sin(ang), 0.0), side*0.07)*len*u;
        float d = raySeg(ro, rd, prev, q, s);
        if (d < best){ best = d; bs = (float(j) - 1.0 + s)/6.0; }
        prev = q;
      }
      float fl = 0.7 + 0.3*sin(uTime*1.3 + fi*1.7 + side);
      acc += (vec3(1.0)*halo(best, max(0.04, px))*0.8*(1.0 - bs) + AR_GOLD*halo(best, 0.15 + 0.4*bs)*0.8)*(1.0 - bs*bs)*fl;
    }
  }
  return acc;
}

// the round-8 effects: the radiant light rising off the blade like a quiet flame
vec3 r8_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, y0 = L*0.2, y1 = L*0.97;
  float sw = clamp(uSwing, 0.0, 1.0);
  vec3 col = fxSmoke(ro, rd, y0, y1, 2.6, 3.0, 2.7, AR_GOLD)*(0.5 + 0.3*sw);
  col += fxSmoke(ro, rd, y0 + 3.0, y1, 1.6, 4.0, 6.1, AR_PALE)*0.3;
  col += fxAura(ro, rd, y0, y1, 2.4, AR_GOLD)*0.1;
  return col*uFx;
}

// the blade: Sol's glow and core from the file, white-gold, with slow waves of light rising
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float wave = pow(0.5 + 0.5*sin(b.y*0.25 - uTime*1.1), 3.0);
  return recol(g, mix(AR_GOLD, AR_PALE, 0.5 + 0.5*wave))*(0.72 + 0.28*wave + 0.3*clamp(uSwing, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*(1.0 + 0.5*clamp(uClash, 0.0, 1.0)); }

// ---- round 9: the big effects layer ----
// Round 11 (Creyon): its own look, Sol's way: its round-8 effects as part 0, its own parts
// (`part(k)`, the `// @part` lines) and Sol's pieces from its file; nothing shared with the others.
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  vec3 col = r8_vol(ro, rd, px)*part(0.0);
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  // round 15 (Creyon): soft rays of light reaching out of the blade and fading, slowly, in
  // place of the ribbons other blades shared with the Aurodium
  fx += fxRays(ro, rd, px, y0, y1, 8.0, 3.6, 0.3, 4.2, AR_GOLD)*(0.9 + 0.4*sw)*part(1.0);
  // round 15 (Creyon): the effects take the wearer's colour with the blade (round 14 kept
  // them gold)
  fx += ar_wings(ro, rd, px)*(0.8 + 0.4*sw)*part(2.0);
  return col + fx*uFx;
}

// round 16 (Creyon: it needs a bit more): four orbs of the Ashla's light circling the blade slowly
// at different heights, each a soft heart trailing light; they circle it, so they are drawn in
// true 3D round it (Sol's way); with its rays (part 1)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  vec3 acc = vec3(0.0);
  if (fxNear(ro, rd, L*0.15, L*0.95) > 5.0) return acc;
  for (int j = 0; j < 4; j++){
    float fj = float(j);
    float best = 1e3, bs = 0.0;
    vec3 prev = vec3(0.0);
    for (int i = 0; i <= 10; i++){
      float tt = t - float(i)*0.1;
      vec3 p = around(2.4 + 0.4*sin(tt*0.6 + fj), tt*0.09 + fj/4.0, L*(0.25 + 0.18*fj) + 0.8*sin(tt*0.5 + fj*2.0));
      if (i > 0){ float d = raySeg(ro, rd, prev, p, s); if (d < best){ best = d; bs = (float(i) - 1.0 + s)/10.0; } }
      prev = p;
    }
    float tail = 1.0 - bs;
    acc += (vec3(1.0)*halo(best, max(0.1, px))*pow(tail, 6.0)*1.8 + AR_GOLD*halo(best, 0.35 + 0.25*bs)*tail*0.9
            + AR_GOLD*halo(best, 1.0)*0.15*tail);
  }
  return acc*uFx;
}
