// Carbonite: a blade that keeps being frozen in carbonite and breaking free. A wave of freezing
// steam climbs from the hilt and seals the blade in a slab of dark bronze carbonite with cast
// grooves and a cold sheen, vapour billowing off it. The trapped light beats through hairline
// cracks that spread until the slab bursts: its shards fly out and float round the blade, slowly
// tumbling for a long while, the freed blade blazing with rays of light; then they fly back and
// fuse into the slab again, which stays solid for a good while as new cracks spread, and bursts
// again. A swing makes the cracks flare; a clash chips shards off the slab.
// @part Carbonite slab
// @part Floating shards
// @part Blaze and steam
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.20,0.45,0.75],"corona.rim_hot":[0.60,0.88,1.0],"corona.inner":[0.92,0.98,1.0],"core.white":[1.0,1.0,1.0],"core.fringe_cool":[0.50,0.80,1.0],"core.fringe_hot":[0.80,0.95,1.0],"light.color":[0.60,0.88,1.0],"trail":[0.60,0.88,1.0]}

const float CB_W = 0.78;      // the slab's half width
const float CB_RR = 0.24;     // how rounded its corners are
const float CB_Y0 = 0.3;      // the slab's foot, above the glowing emitter
const float CB_P = 18.0;      // seconds per hold, break, float and fuse (Creyon's round 9: slower)
float cb_ph(){ return mod(uTime - 0.5, CB_P); }
float cb_btop(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
// the freezing front climbing from the hilt
// (on ignition only: after that the slab never vanishes, it bursts and fuses again)
float cb_front(){ return -0.5 + (uLength + 2.5)*smoothstep(0.0, 1.6, uAge); }
// 1 while the slab stands
float cb_held(){ float ph = cb_ph(); return ph < 5.0 || ph > 12.2 ? 1.0 : 0.0; }
// how solid the slab is: whole while held, fading in over the returning shards as they fuse
float cb_slabA(){ float ph = cb_ph(); return cb_held()*(ph > 12.2 ? smoothstep(12.2, 12.6, ph) : 1.0); }
// the shards' flight: 0 on the blade, 1 out on their orbit (they burst out, float round the
// blade, then fly back and fuse into the slab again)
float cb_out(){ float ph = cb_ph(); return smoothstep(5.0, 5.8, ph)*(1.0 - smoothstep(11.4, 12.4, ph)); }
float cb_shards(){ float ph = cb_ph(); return step(5.0, ph)*step(ph, 12.6); }
// the slab's top
float cb_stop(){ return min(min(cb_front(), uLength + 0.5), cb_btop() + 0.3); }
// how far the cracks have spread
float cb_crack(){ float ph = cb_ph(); return clamp(0.15 + smoothstep(1.5, 4.9, ph)*0.85*step(ph, 5.0) + 0.6*uSwing + uClash, 0.0, 1.3); }
// the burst: 0 at the break .. 1 when the shards are gone
float cb_burst(){ return clamp((cb_ph() - 5.0)/1.5, 0.0, 1.0); }
float cb_beat(){ return 0.55 + 0.45*pow(0.5 + 0.5*sin(uTime*2.6), 10.0) + 0.35*pow(0.5 + 0.5*sin(uTime*2.6 - 0.6), 14.0); }
// the light beating through the hairline cracks at (across, along) on the slab
float cb_cracks(vec2 f, float px){
  vec2 cell;
  float e = voro(vec2(f.x*2.4, f.y*0.75), cell);
  float reach = cb_crack();
  float keep = step(h21(cell + 3.1), reach);
  return halo(e, max(0.045 + 0.05*reach, px*1.2))*keep;
}
// the slab, drawn over Sol's glow and core as his solid blades are (the Comic's ink, the
// Quicksilver's chrome): a rounded bar of dark bronze facing the camera, lit from one side,
// with cast grooves across it, a raised rib, a cold sheen and the trapped light in its cracks
vec4 base_solid(vec4 o, vec2 b, float px){
  if (uFx < 0.01 || cb_held() < 0.5) return o;
  float top = cb_stop();
  if (top < CB_Y0 + 0.2) return o;
  float t = uTime, L = uLength;
  float sh = smoothstep(4.4, 5.0, cb_ph())*step(cb_ph(), 5.0)*0.05*sin(uTime*70.0);
  float w = CB_W + 0.06*(vn(vec2(b.y*0.7, 3.0)) - 0.5);
  w *= 1.0 - 0.35*smoothstep(L - 1.5, L + 0.5, b.y);
  float x = b.x + sh;
  // a rounded end at the foot and the top
  float dy = max(max(b.y - (top - CB_RR), (CB_Y0 + CB_RR) - b.y), 0.0);
  float d = length(vec2(max(abs(x) - (w - CB_RR), 0.0), dy)) - CB_RR;
  float cov = fillD(d, px);
  if (cov <= 0.0) return o;
  float u = x/w;
  float nz = fbm3(vec2(u*2.6 + 7.0, b.y*0.9));
  float gy = abs(fract(b.y/4.0) - 0.5)*4.0;
  float groove = 1.0 - 0.55*halo(gy, 0.12);
  float lip = halo(gy - 0.18, 0.08)*0.5;
  float rib = halo(abs(abs(u) - 0.55), 0.06)*0.35;
  // a bar's shading: the light from the upper left, a bright edge, darker toward the far side
  float nx = clamp(u, -1.0, 1.0);
  float dif = clamp(0.55 - 0.5*nx + 0.15*sqrt(1.0 - nx*nx), 0.0, 1.0);
  float spec = pow(halo(nx + 0.45, 0.22), 2.0);
  float fres = pow(abs(nx), 4.0);
  vec3 rgb = vec3(0.34, 0.27, 0.19)*(0.55 + 0.7*(nz - 0.35))*groove*(0.25 + 0.95*dif);
  rgb += vec3(0.55, 0.48, 0.38)*(lip + rib)*(0.4 + 0.6*dif);
  rgb += vec3(0.75, 0.82, 0.9)*spec*0.4*(0.6 + 0.4*nz);
  float sheen = halo(b.y - mod(t*6.0, L + 20.0) + 10.0 - u*2.0, 1.4);
  rgb += vec3(0.45, 0.6, 0.7)*sheen*(0.25 + 0.5*fres) + vec3(0.35, 0.5, 0.6)*fres*0.3;
  rgb = mix(rgb, vec3(0.78, 0.88, 0.94)*(0.6 + 0.4*nz), halo(b.y - cb_front(), 1.3)*0.7*smoothstep(2.6, 1.4, uAge));
  float glowIn = (0.5 + 1.5*cb_beat())*(0.5 + 0.9*cb_crack());
  rgb += vec3(0.75, 0.95, 1.0)*cb_cracks(vec2(u, b.y), px)*glowIn*1.2;
  rgb += vec3(0.25, 0.55, 0.8)*halo(u, 0.5)*0.12*glowIn*smoothstep(0.4, 1.2, cb_crack());
  cov *= cb_slabA();
  // reforming: the slab glows where the shards fused
  rgb += vec3(0.7, 0.92, 1.0)*(1.0 - smoothstep(12.3, 13.1, cb_ph()))*step(12.3, cb_ph())*0.8;
  return vec4(mix(o.rgb, rgb, cov), max(o.a, cov));
}
// a flying shard of slab: how much it covers the ray; th its depth, sh.x its facet shade,
// sh.y the glowing broken edge
float cb_shard(vec3 ro, vec3 rd, vec3 c, float sz, float spin, float seed, float px, out float th, out vec2 sh){
  float d = rayPt(ro, rd, c, th);
  sh = vec2(0.0);
  if (d > sz*2.0 + px) return 0.0;
  vec3 o = ro + rd*th - c;
  vec2 a = rot(spin + seed*6.0)*o.xy; o = vec3(a, o.z);
  vec2 b = rot(spin*0.7 + seed*3.0)*o.yz; o = vec3(o.x, b);
  // an irregular broken triangle, thick as a slab plate
  float j = h11(seed*3.7);
  float tri = max(max(dot(o.xy, vec2(0.0, 1.0))*(0.8 + 0.5*j), dot(o.xy, vec2(0.87, -0.5))), dot(o.xy, vec2(-0.87 + 0.3*j, -0.5)));
  float k = max(tri*1.25, abs(o.z)*2.2 + abs(o.x)*0.15) - sz*0.75;
  sh = vec2(0.5 + 0.5*h21(sign(o.xy) + vec2(seed*7.0, sign(o.z))), halo(abs(k), 0.035 + px*0.8));
  return clamp(0.5 - k/max(px, 1e-3), 0.0, 1.0);
}
// the burst and the clash: shards of slab, each a piece facing the camera, flung out in 3D
// round 10: the shards float round the blade, so they are drawn in true 3D (only orbits are)
vec4 base_orbit_solid(vec3 ro, vec3 rd, float px){
  float L = uLength;
  vec4 col = vec4(0.0);
  if (uFx < 0.01) return col;
  // the shards: burst out, float round the blade tumbling slowly, then fly back and fuse
  if (cb_shards() > 0.5){
    float ou = cb_out(), ph = cb_ph();
    float bestT = 1e5;
    for (int i = 0; i < 30; i++){
      float fi = float(i);
      float a0 = h11(fi*1.7 + 0.3);
      float y0 = (0.03 + 0.94*fract(fi*0.381 + 0.07))*L;
      float R = 2.6 + 3.0*h11(fi*5.1);
      // they drift round the blade while out, each at its own pace
      float drift = (ph - 5.0)*(0.035 + 0.04*h11(fi*2.9))*ou;
      vec3 c = around(0.55 + (R - 0.55)*ou, a0 + drift, y0 + 1.5*(h11(fi*7.3) - 0.5)*ou + 0.5*sin(ph*0.8 + fi)*ou);
      float sz = (0.6 + 0.45*h11(fi*9.1))*(0.35 + 0.65*ou);
      float th; vec2 sh;
      float cv = cb_shard(ro, rd, c, sz, ph*(0.6 + 0.8*h11(fi)) + fi, fi, px, th, sh);
      if (cv > 0.0 && th < bestT){
        bestT = th;
        vec3 rgb = vec3(0.30, 0.24, 0.17)*sh.x*1.2 + vec3(0.7, 0.92, 1.0)*sh.y*(0.6 + 1.0*(1.0 - ou));
        col = vec4(rgb, cv);
      }
    }
  }
  return col;
}
// a clash chips shards off the slab round the blade
vec4 base_vol_solid(vec3 ro, vec3 rd, float px){
  float L = uLength;
  vec4 col = vec4(0.0);
  if (uFx < 0.01) return col;
  if (uClash > 0.01 && cb_held() > 0.5){
    float a = 1.0 - uClash;
    for (int i = 0; i < 6; i++){
      float fi = float(i);
      vec3 c = vec3(0.0, L*0.6, 0.0) + vec3(sin(fi*2.4), 0.6*(h11(fi*3.0) - 0.4), cos(fi*2.4))*(0.8 + 4.5*a);
      float th; vec2 sh;
      float cv = cb_shard(ro, rd, c, 0.4*(1.0 - a*0.6), a*9.0, fi + 40.0, px, th, sh);
      if (cv > 0.0) col = vec4(mix(col.rgb, vec3(0.32, 0.26, 0.19)*sh.x + vec3(0.6, 0.9, 1.0)*sh.y*1.5*uClash, cv), max(col.a, cv));
    }
  }
  return col;
}
// cold vapour: soft puffs rolling off the slab and climbing with the freezing front, blown
// out in a ring when it bursts; each puff faces the camera
vec3 cb_steam(vec3 ro, vec3 rd){
  float t = uTime, L = uLength, ta;
  float held = cb_held(), bu = cb_burst(), ph = cb_ph(), front = cb_front();
  vec3 col = vec3(0.0);
  for (int i = 0; i < 22; i++){
    float fi = float(i);
    float f = fract(t*(0.18 + 0.1*h11(fi*2.1)) + h11(fi*5.7));
    float y0 = h11(fi*3.3 + floor(t*(0.18 + 0.1*h11(fi*2.1)) + h11(fi*5.7)))*min(cb_stop(), cb_btop());
    float r = 0.9 + 1.6*f;
    float yy = y0 + 1.5*f;
    // at the freezing front the steam billows thick; when it bursts it is blown out
    float wave = smoothstep(2.6, 1.6, uAge);
    yy = mix(yy, front - 1.0 + 2.0*h11(fi*1.3), wave*step(fi, 9.0));
    r += 3.5*bu*(1.0 - bu)*4.0*step(0.001, bu);
    vec3 p = around(r, h11(fi*8.9) + 0.05*t, yy);
    float dd = rayPt(ro, rd, p, ta);
    if (dd > 3.0) continue;
    float life = sin(3.1416*f)*(held*0.8 + (1.0 - held)*0.4) + wave*0.6*step(fi, 9.0);
    col += vec3(0.72, 0.86, 0.95)*(halo(dd, 0.55 + 0.6*f)*0.22 + halo(dd, 1.3)*0.05)*life;
  }
  return col;
}
vec3 base_vol(vec3 ro, vec3 rd, float px){
  float L = uLength;
  float bu = cb_burst();
  vec3 c = vec3(0.6, 0.88, 1.0);
  vec3 light = vec3(0.0);
  if (uFx < 0.01) return light;
  float ta;
  // the burst: the freed blade's blaze
  if (bu > 0.0 && bu < 1.0){
    float dAx = raySegRaw(ro, rd, vec3(0.0, CB_Y0, 0.0), vec3(0.0, cb_btop(), 0.0), ta);
    light += c*(halo(dAx, 0.9)*1.2 + halo(dAx, 2.5)*0.25)*(1.0 - bu);
  }
  // clash: a ring of light round the struck point, and a flash
  if (uClash > 0.01){
    float a = 1.0 - uClash;
    vec3 h = rayAt(ro, rd, L*0.6);
    if (h.z > 0.0) light += vec3(0.75, 0.95, 1.0)*halo(hideD(abs(length(h.xy) - 0.8 - 6.0*a), behindBlade(ro, rd, h.z)), 0.25)*1.4*uClash;
    float dc = rayPtRaw(ro, rd, vec3(0.0, L*0.6, 0.0), ta);
    light += vec3(0.75, 0.95, 1.0)*halo(dc, 1.4)*pow(uClash, 2.0);
  }
  // round 9: free, the blade blazes with rays and sparks of cold light; the burst and the fuse
  // each throw a shock ring; held, light leaks from the cracks in thin beams
  float y0 = L*0.2, y1 = L*0.97;
  float ph = cb_ph(), free_ = cb_shards()*cb_out();
  vec3 ice = vec3(0.75, 0.95, 1.0);
  light += fxSparks(ro, rd, px, y0, y1, 20.0, 6.0, 0.9, 1.9, ice)*free_;
  light += fxArcs(ro, rd, px, y0, y1, 4.0, 4.0, 1.5, 6.6, c)*free_;
  light += fxAura(ro, rd, y0, y1, 3.0, c)*(0.3 + 0.9*free_);
  light += fxShock(ro, rd, px, L*0.5, (ph - 5.0)/1.6, 10.0, ice)*1.6;
  light += fxShock(ro, rd, px, L*0.5, (ph - 12.3)/1.2, 7.0, ice);
  return light + cb_steam(ro, rd);
}
// the blade: Sol's glow and core from the file, blazing as it breaks free
float cb_flare(){ float ph = cb_ph(); return 1.0 + 0.9*exp(-max(ph - 5.0, 0.0)*1.5)*step(5.0, ph)*step(ph, 12.4) + 0.3*cb_shards(); }
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return g*cb_flare(); }
vec3 sig_core(vec3 c, vec2 b, float px){ return c*cb_flare(); }
// round 11: sig_solid gated by part 0
vec4 sig_solid(vec4 o, vec2 b, float px){
  if (part(0.0) < 0.5) { return o; }
  return base_solid(o, b, px);
}
// round 11: sig_orbit_solid gated by part 1
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5) { return vec4(0.0); }
  return base_orbit_solid(ro, rd, px);
}
// round 11: sig_vol_solid gated by part 0
vec4 sig_vol_solid(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec4(0.0); }
  return base_vol_solid(ro, rd, px);
}
// round 11: sig_vol gated by part 2
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec3(0.0); }
  return base_vol(ro, rd, px);
}
