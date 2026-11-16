// Mustafar Eruption: the lava rivers of Mustafar as a blade. Molten rock pours down it in bright
// tongues that cool to dark crust and crack open again; lava falls run down both sides and drip off
// it, flames lick up its length, lava fountains throw glowing drops that fall back, dark ash smoke
// billows up, ash drifts down, embers rain and the air shimmers with heat. A clash flares it.
// @part Lava falls
// @part Fire and fountains
// @part Ash smoke and ash
// @room 8
// @palette {"corona.rim_cool":[0.5,0.06,0.0],"corona.rim_hot":[1.0,0.4,0.05],"corona.inner":[1.0,0.75,0.4],"core.white":[1,0.92,0.75],"core.fringe_cool":[1.0,0.35,0.05],"core.fringe_hot":[1.0,0.65,0.3],"light.color":[1.0,0.4,0.08],"trail":[1.0,0.4,0.08]}

vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  float flow = fbm3(vec2(b.x*2.0, b.y*0.22 + uTime*0.9));
  float crust = smoothstep(0.5, 0.66, fbm3(vec2(b.x*1.5 + 3.1, b.y*0.12 + uTime*0.25)));
  float tongue = pow(0.5 + 0.5*sin(b.y*0.35 + uTime*3.2 + 2.0*flow), 3.0)*part(0.0);
  vec3 lava = vec3(1.25, 0.62, 0.22)*(0.8 + 0.5*flow) + vec3(0.4, 0.25, 0.1)*tongue;
  vec3 rock = vec3(0.32, 0.12, 0.07) + vec3(0.9, 0.3, 0.05)*halo(abs(crust - 0.5), 0.12);
  return g*mix(lava, rock, crust*0.75);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 fire = vec3(1.0, 0.45, 0.1), hot = vec3(1.0, 0.75, 0.35);
  float s;
  // lava falls (round 15: straight runnels pouring down close to the blade, in place of the
  // wavy streams it shared with the Aurodium): bright heads of molten rock running down, a
  // glowing trail behind each cooling to dark red
  if (fxNear(ro, rd, y0, y1) < 3.0){
    for (int i = 0; i < 8; i++){
      float fi = float(i);
      float side = mod(fi, 2.0) < 0.5 ? -1.0 : 1.0;
      float x = side*(0.55 + 0.35*h11(fi*4.1));
      float cyc = t*(0.3 + 0.15*h11(fi*2.3)) + h11(fi*7.9);
      float q = fract(cyc);
      float yh = mix(y1, y0 - 2.0, q*q*(3.0 - 2.0*q));
      float yt = min(yh + 6.0 + 4.0*h11(fi*3.3), y1);
      float sl;
      // round 19: all round the blade, in three dimensions
      float d = raySeg(ro, rd, turnY(vec3(x, yh, 0.0), h11(fi*5.7)*0.5), turnY(vec3(x + side*0.15, yt, 0.0), h11(fi*5.7)*0.5), sl);
      vec3 c = mix(vec3(1.0, 0.85, 0.45), vec3(0.75, 0.12, 0.02), sl);
      fx += (c*halo(d, max(0.07 - 0.04*sl, px))*1.8 + fire*halo(d, 0.35)*0.35*(1.0 - sl))*(1.0 - smoothstep(0.85, 1.0, q))*0.8*part(0.0);
    }
  }
  // molten drops dripping off and falling past the hilt
  fx += fxFall(ro, rd, px, y0, y1, 1.8, 16.0, 7.0, 0.9, 0.0, 6.1, hot)*0.9*part(0.0);
  // flames licking up the blade, thin and bright near it
  fx += fxSmoke(ro, rd, y0, y1, 2.0, 7.0, 4.7, vec3(1.0, 0.42, 0.08))*1.5*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 1.1, 9.0, 2.2, vec3(1.0, 0.7, 0.3))*1.1*part(1.0);
  fx += fxSparks(ro, rd, px, y0, y1, 6.0 + 8.0*sw, 4.5, 0.6, 7.7, fire)*0.8*part(1.0);
  // lava fountains: molten drops thrown up off the blade, falling back in arcs
  float lta;
  if (fxNear(ro, rd, y0, y1) < 6.0){
    for (int i = 0; i < 12; i++){
      float fi = float(i);
      float cyc = t*0.45*(0.8 + 0.4*h11(fi*2.9)) + h11(fi*7.7);
      float k = floor(cyc), q = fract(cyc);
      float y = mix(y0, y1 - 6.0, h11(k*3.1 + fi));
      float side = h11(k*5.3 + fi) < 0.5 ? -1.0 : 1.0;
      float tt = q*1.6;
      vec3 p = turnY(vec3(side*(0.5 + 2.2*tt), y + 6.0*tt - 4.5*tt*tt, 0.0), h11(k*7.1 + fi)*0.5);
      float d = rayPt(ro, rd, p, lta);
      fx += mix(vec3(1.0, 0.85, 0.5), fire, q)*(halo(d, max(0.12, px))*1.2 + halo(d, 0.5)*0.2)*(1.0 - q)*part(1.0);
    }
  }
  // dark ash smoke billowing up, ash flakes drifting down
  fx += fxSmoke(ro, rd, y0, y1, 3.8, 2.0, 1.4, vec3(0.3, 0.2, 0.18))*0.8*part(2.0);
  fx += fxSmoke(ro, rd, y0, y1, 4.0, 2.4, 8.2, vec3(0.5, 0.3, 0.22))*0.35*part(2.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 3.5, 12.0, 1.8, 0.2, 0.8, 1.5, 0.6, 3.6, vec3(0.5, 0.45, 0.42))*0.7*part(2.0);
  return fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 65 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.65; }
