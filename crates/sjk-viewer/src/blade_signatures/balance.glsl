// Balance: the Force in balance, the light and the dark held level in one blade (Creyon's round
// 16 rework: he did not like the scales). A grey blade with two faces, a warm white-gold one and a
// violet one, turning slowly round it, so the light and the dark take turns facing you; bands of
// light climb it while bands of shadow sink through them, and every few seconds a wave of light
// from the hilt and a wave of shadow from the tip meet at its middle in a white flash, the point of
// balance, neither winning. Round it, motes of light rise and a slow violet smoke sinks; a sun of
// light and an orb of shadow circle it opposite each other, never meeting. A swing brightens all;
// a clash flares the blade white.
// @part The light
// @part The dark
// @part Point of balance
// @room 10
// @solid 1

const vec3 BA_LIGHT = vec3(1.0, 0.86, 0.5);
const vec3 BA_DARK = vec3(0.5, 0.22, 0.95);
const float BA_P = 5.0;
// the waves' cycle, 0 to 1: they meet at the blade's middle at 0.5
float ba_q(){ return fract(uTime/BA_P); }
float ba_meet(){ return exp(-pow((ba_q() - 0.5)/0.035, 2.0)); }

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  float L = uLength;
  // the two faces turning: the line between them sweeps across, and as it leaves one edge the
  // colours change sides, as if the blade turned on its axis
  float turn = uTime*0.35;
  float edge = cos(turn)*0.3;
  float facing = sin(turn) > 0.0 ? 1.0 : -1.0;
  float side = smoothstep(-0.06, 0.06, (b.x - edge)*facing);
  vec3 c = mix(BA_LIGHT, BA_DARK, side);
  c = mix(c, vec3(0.92, 0.92, 0.98), exp(-pow(b.x/0.05, 2.0))*0.35);
  // bands of light climbing, bands of shadow sinking through them
  float up = pow(0.5 + 0.5*sin(b.y*0.35 - uTime*2.0), 6.0)*part(0.0);
  float down = pow(0.5 + 0.5*sin(b.y*0.35 + uTime*2.0 + 1.5), 6.0)*part(1.0);
  c = c*(1.0 + 0.6*up - 0.4*down) + BA_LIGHT*up*0.3;
  // a wave of light from the hilt and one of shadow from the tip, meeting at the middle in a flash
  float q = ba_q();
  float go = smoothstep(0.1, 0.5, q)*step(q, 0.5);
  float yl = L*0.5*go, yd = L - L*0.5*go;
  vec3 waves = (BA_LIGHT*halo(b.y - yl, 1.2) + BA_DARK*halo(b.y - yd, 1.2)*1.3)*step(0.1, q)*step(q, 0.5);
  vec3 flash = vec3(1.0)*ba_meet()*halo(b.y - L*0.5, 3.0)*1.2;
  return l*(c + (waves*0.9 + flash)*part(2.0))*(1.0 + 0.3*clamp(uSwing, 0.0, 1.0))
         + vec3(1.0)*l*0.6*clamp(uClash, 0.0, 1.0);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }

// the effects round the blade, in three dimensions: motes of light rising, violet smoke sinking,
// the flash at the point of balance
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, sw = clamp(uSwing, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 18.0, 0.05, 2.1, BA_LIGHT)*(0.8 + 0.4*sw)*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.2, -1.0, 4.4, BA_DARK*0.7)*1.2*part(1.0);
  fx += fxFlare(ro, rd, vec3(0.0, L*0.5, 0.0), px, 0.3, 2.5 + 4.0*ba_meet(), 4.0, 0.785, vec3(0.95, 0.9, 1.0))*ba_meet()*part(2.0);
  return fx*uFx;
}

// where the sun of light and the orb of shadow are: opposite each other round the blade, rising
// and sinking together
vec3 ba_sun(){ float L = uLength; return around(2.6, uTime*0.12, L*(0.55 + 0.2*sin(uTime*0.4))); }
vec3 ba_shade(){ float L = uLength; return around(2.6, uTime*0.12 + 0.5, L*(0.55 + 0.2*sin(uTime*0.4))); }

// they circle the blade, so they are drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  float L = uLength, ta;
  vec3 fx = vec3(0.0);
  // round 23: past the sun's and the orb's widest glow (2.6 out, glow 2 wide), which ended at a
  // hard edge
  if (fxNear(ro, rd, L*0.2, L*0.9) > 8.0) return fx;
  float ds = rayPt(ro, rd, ba_sun(), ta);
  fx += (vec3(1.0)*halo(ds, max(0.3, px))*2.0 + BA_LIGHT*halo(ds, 0.8)*1.2 + BA_LIGHT*halo(ds, 2.0)*0.3)*part(0.0)*bladeSeen(ro, rd, ba_sun());
  float dd = rayPt(ro, rd, ba_shade(), ta);
  // the orb of shadow's violet rim and glow (its dark heart is the solid layer)
  fx += (BA_DARK*halo(abs(dd - 0.6), 0.1)*2.0 + BA_DARK*halo(dd, 2.0)*0.35)*part(1.0)*bladeSeen(ro, rd, ba_shade());
  return fx*uFx;
}

// the orb of shadow's dark heart, over what is behind it
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5 || uFx < 0.01) return vec4(0.0);
  float ta;
  float dd = rayPt(ro, rd, ba_shade(), ta);
  return vec4(vec3(0.02, 0.0, 0.05), fillD(dd - 0.56, px)*0.92*bladeSeen(ro, rd, ba_shade()));
}
