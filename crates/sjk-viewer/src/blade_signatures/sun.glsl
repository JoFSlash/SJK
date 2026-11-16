// Sun: Sol's own blade, blade 001, a star held in the hand. A white-gold core under an orange
// corona that breathes slowly, calm and huge where the Starforge is busy: dark sunspots drift
// along the core, more prominences than any of Sol's blades arch off it, a solar wind of pale motes
// rides out to the tip, and the widest warm glow of any blade. Coronal rain: faint magnetic
// loops stand off the blade, and plasma cooling at their tops slides back down them to the
// blade in glowing beads, as on the real Sun. It lights as a sunrise, deep red at first and
// warming to white gold as it grows, with a dawn flare at the emitter once it is out; it goes
// out as a sunset, cooling to deep red as it slides back in. A swing makes the corona swell
// and the wind blow harder; a clash is a total eclipse's diamond ring: a dark moon covers the
// blade where it lands, the corona shows as a ring round it and, as the moon slides off, one
// blinding point flares on the ring's edge.
// @part Sunspots and corona
// @arcs Prominences
// @motes Sparks
// @star Glint
// @haze Haze
// @room 10
// @moments own
// @palette {"corona.rim_cool":[1.0,0.2,0.02],"corona.rim_hot":[1.0,0.42,0.05],"corona.inner":[1.0,0.72,0.24],"core.white":[1.0,0.97,0.86],"core.fringe_cool":[1.0,0.42,0.05],"core.fringe_hot":[1.0,0.72,0.24],"light.color":[1.5,0.85,0.3],"trail":[1.0,0.6,0.12]}
// @trail plasma
// @extras {"card":"sjk_alpha_000","card_name":"SJK Alpha 000 (Sol)","finish":"solar"}
// @extra {"id":"flare_trail","name":"Flare trail","trail":"flare"}
// @extra {"id":"swing_embers","name":"Swing embers","sheds":{"kind":"ember","per_unit":0.1,"life":0.9,"size":0.8,"fall":0.0,"rise":36.0,"drift":10.0,"fling":0.16}}
// @extra {"id":"heat_wake","name":"Heat wake","wake":true}


const vec3 SU_WHITE = vec3(1.0, 0.96, 0.84);
const vec3 SU_GOLD = vec3(1.0, 0.7, 0.26);
const vec3 SU_ORANGE = vec3(1.0, 0.42, 0.06);
const vec3 SU_RED = vec3(1.0, 0.16, 0.03);

// the slow breath of the whole star (about five seconds), 0 to 1
float su_breath(){ return 0.5 + 0.5*sin(uTime*1.25 + uSeed*6.2832); }
// how far the sunrise has warmed (0 deep red, 1 white gold) and the sunset cooled (1 deep red)
float su_dawn(){ return uAge >= 2000.0 ? 1.0 : smoothstep(0.0, 0.9, uAge); }
float su_dusk(){ return uAge >= 2000.0 ? smoothstep(0.0, 0.3, uAge - 2000.0) : 0.0; }
// a colour seen at sunrise or sunset: its level kept, its hue a red sun low on the horizon
vec3 su_sky(vec3 c){
  vec3 low = lvl(c)*mix(SU_RED, SU_ORANGE, 0.35);
  float red = max(1.0 - su_dawn(), su_dusk());
  return mix(c, low, red*0.85);
}
// sunspots at height y: dark umbrae drifting slowly along the blade, 0 to 1
float su_spot(float y){
  float n = vn(vec2(y*0.3 - uTime*0.11, uSeed*7.0 + 3.0));
  float n2 = vn(vec2(y*0.55 + uTime*0.07, uSeed*3.0 + 11.0));
  float spot = max(smoothstep(0.7, 0.86, n), smoothstep(0.76, 0.9, n2)*0.8)*part(0.0);
  return spot*smoothstep(1.5, 4.0, y)*(1.0 - smoothstep(uLength - 3.0, uLength, y));
}
// where a clash lands on the blade
float su_clash_y(){ return uLength*0.62; }

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float br = su_breath();
  float top = uLength*uIgn;
  float cl = clamp(uClash, 0.0, 1.0);
  // the corona breathes and swells with a swing; a clash lights the place it lands
  vec3 o = g*(0.92 + 0.16*br + 0.3*uSwing);
  // the sunspots darken the glow next to the core too, so they show from afar
  o = mix(o, o*vec3(1.0, 0.45, 0.15)*0.6, su_spot(b.y)*0.7*exp(-pow(b.x/max(r*0.8, px), 2.0)));
  // under the moon of the diamond ring the blade goes dark
  float under = 1.0 - smoothstep(SU_MOON*0.9, SU_MOON*1.02, length(b - su_moon_at()));
  o *= 1.0 - 0.97*under*su_moon();
  // the rising sun: a white-hot edge climbing the blade while it grows
  float rising = 1.0 - smoothstep(0.7, 1.0, uIgn);
  o += SU_WHITE*lvl(g)*1.6*exp(-pow((b.y - top)/2.2, 2.0))*rising;
  return su_sky(o);
}

vec3 sig_core(vec3 c, vec2 b, float px){
  // sunspots: the core runs deep orange where they drift past
  vec3 o = mix(c, c*vec3(1.0, 0.42, 0.12)*0.7, su_spot(b.y)*0.85);
  o *= 0.95 + 0.1*su_breath();
  float under = 1.0 - smoothstep(SU_MOON*0.9, SU_MOON*1.02, length(b - su_moon_at()));
  o *= 1.0 - 0.97*under*su_moon();
  return su_sky(o);
}

// coronal rain: loop k (0 to 2) of magnetic field standing off the blade, each living about
// seven seconds and reborn elsewhere; plasma beads slide down it from its top to both feet,
// faster as they fall, glowing gold and cooling to red as they near the blade
vec3 su_rain(vec3 ro, vec3 rd, float px, float k){
  float s, ta;
  vec3 col = vec3(0.0);
  float per = 6.5 + 1.5*h11(k*3.7);
  float cyc = uTime/per + h11(k*5.1 + uSeed);
  float n = floor(cyc), q = fract(cyc);
  float life = smoothstep(0.0, 0.15, q)*(1.0 - smoothstep(0.8, 1.0, q));
  float span = 4.0 + 3.0*h11(n*2.3 + k);
  float y0 = uLength*(0.12 + 0.62*h11(n*1.7 + k*4.1));
  float H = 2.6 + 1.8*h11(n*3.1 + k);
  // its own way round the blade, so from any side some loops show their arch
  float side = h11(n*4.9 + k*2.2);
  // skipped between lives and where the ray passes far from it: the arch and its beads lie within
  // H/2 of the line halfway up it, and their widest halo is nothing past three widths
  if (life <= 0.0) return col;
  float far = raySegRaw(ro, rd, around(0.35 + 0.5*H, side, y0), around(0.35 + 0.5*H, side, y0 + span), s);
  if (far > 0.5*H + 3.0*max(0.6, px)) return col;
  // the loop itself: a faint arch, brightest at its top
  float best = 1e3, bs = 0.0;
  vec3 prev = around(0.35, side, y0);
  for (int j = 1; j <= 10; j++){
    float f = float(j)/10.0;
    vec3 p = around(0.35 + H*sin(3.1416*f), side, y0 + span*f);
    float d = raySeg(ro, rd, prev, p, s);
    if (d < best){ best = d; bs = (float(j) - 1.0 + s)/10.0; }
    prev = p;
  }
  float top = sin(3.1416*bs);
  col += (SU_ORANGE*halo(best, max(0.08, px))*0.55 + SU_RED*halo(best, 0.6)*0.2)*(0.4 + 0.6*top);
  // the beads: each starts at the top and falls to one foot, accelerating
  for (int i = 0; i < 6; i++){
    float fi = float(i);
    float g = fract(uTime*(0.22 + 0.08*h11(fi*1.9 + k)) + h11(fi*6.7 + k*3.3));
    float fall = g*g;
    float f = 0.5 + (h11(fi*2.9 + k) < 0.5 ? -0.5 : 0.5)*fall;
    vec3 p = around(0.35 + H*sin(3.1416*f), side, y0 + span*f);
    float d = rayPt(ro, rd, p, ta);
    float bead = smoothstep(0.0, 0.1, g)*(1.0 - smoothstep(0.85, 1.0, g));
    vec3 c = mix(SU_GOLD, SU_RED, smoothstep(0.2, 1.0, fall));
    col += (mix(SU_WHITE, c, 0.5)*halo(d, max(0.09, px))*1.6 + c*halo(d, 0.4)*0.35)*bead;
  }
  return col*life;
}

// the diamond ring's moon: radius in units, and how much of it covers the blade (0 to 1)
const float SU_MOON = 2.4;
float su_moon(){
  float a = 1.0 - clamp(uClash, 0.0, 1.0);
  return uClash > 0.01 ? smoothstep(0.0, 0.06, a)*(1.0 - smoothstep(0.35, 0.75, a)) : 0.0;
}
// where the moon's disc is in the blade's picture: it slides up the blade over the clash
vec2 su_moon_at(){
  float a = 1.0 - clamp(uClash, 0.0, 1.0);
  return vec2(0.0, su_clash_y() - 0.6 + 1.6*a);
}

vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, br = su_breath();
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.12, y1 = L*0.96;
  vec3 col = vec3(0.0);
  float corona = part(0.0)*uFx;

  // the widest warm glow of any blade, breathing with the star
  col += fxAura(ro, rd, y0, y1, 4.6 + 0.6*br + 1.2*sw, SU_ORANGE)*(0.7 + 0.25*br)*corona;
  // the solar wind: pale motes riding out along the blade to the tip
  col += fxMotes(ro, rd, px, y0, y1 + 1.0, 0.9, 4.2, 16.0, 0.07 + 0.05*sw, 1.9, mix(SU_GOLD, SU_WHITE, 0.4))*(0.8 + 0.6*sw)*corona;

  // the dawn: once the blade is out, a flare at the emitter and a warm bloom up the blade
  float dawn = smoothstep(0.25, 0.45, uAge)*(1.0 - smoothstep(0.6, 1.8, uAge))*step(uAge, 1000.0);
  if (dawn > 0.001){
    col += fxFlare(ro, rd, vec3(0.0, 1.2, 0.0), px, 0.45, 9.0, 4.0, 0.4, SU_GOLD)*dawn*1.4;
    col += fxAura(ro, rd, 0.0, L, 2.0 + 6.0*smoothstep(0.25, 1.4, uAge), mix(SU_WHITE, SU_GOLD, 0.5))*dawn*0.8;
  }

  // coronal rain on three loops (none drawn with the part off)
  if (corona > 0.0){
    for (int k = 0; k < 3; k++){
      col += su_rain(ro, rd, px, float(k))*corona;
    }
  }

  // the clash: a total eclipse's diamond ring
  float moon = su_moon();
  if (moon > 0.001){
    float a = 1.0 - clamp(uClash, 0.0, 1.0);
    vec2 m = su_moon_at();
    float ta;
    // the moon stands on the blade's side facing the eye, so the blade never hides it
    vec2 eye = normalize(ro.xz + vec2(1e-4, 0.0));
    vec3 mc = vec3(eye.x*0.6, m.y, eye.y*0.6);
    float dm = rayPtRaw(ro, rd, mc, ta);
    // the moon's disc hides the aura, the rain and the wind behind it
    col *= 1.0 - smoothstep(SU_MOON*1.02, SU_MOON*0.75, dm)*moon;
    // the corona round the moon: a thin bright ring and a soft streaming glow outside it
    float streams = 0.7 + 0.3*vn(vec2(atan(rd.y, rd.x)*3.0, a*2.0));
    col += (SU_WHITE*halo(abs(dm - SU_MOON), max(0.12, px))*1.6 + mix(SU_WHITE, SU_GOLD, 0.5)*halo(max(dm - SU_MOON, 0.0), 1.4)*0.7*streams)*moon;
    // the diamond: one blinding point on the ring's edge as the moon slides off
    float diamond = exp(-pow((a - 0.32)/0.11, 2.0));
    vec3 at = mc + vec3(-eye.y, 0.0, eye.x)*SU_MOON*0.7 - vec3(0.0, SU_MOON*0.7, 0.0);
    col += fxFlare(ro, rd, at, px, 0.35, 11.0, 4.0, 0.3, SU_GOLD)*diamond*2.2;
  }
  return su_sky(col);
}
