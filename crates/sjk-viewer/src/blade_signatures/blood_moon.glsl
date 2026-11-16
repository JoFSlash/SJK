// Dathomir Blood Moon: the Nightsisters' red night as a blade. A dark crimson blade with a shadowed
// heart and a slow red tide climbing it; drops of red light drip off it and fall, a red mist curls up
// off it, two dark red streams wind up beside it, a blood-red moon circles it slowly in 3D with a
// shadow sliding across its face, and shadow copies trail every swing (round 16). A clash flares it.
// @part Blood drips
// @part Red mist
// @part Blood moon
// @room 8
// @solid 1
// @palette {"corona.rim_cool":[0.35,0.0,0.02],"corona.rim_hot":[1.0,0.12,0.08],"corona.inner":[0.25,0.0,0.02],"core.white":[0.55,0.05,0.05],"core.fringe_cool":[0.6,0.02,0.02],"core.fringe_hot":[1.0,0.2,0.15],"light.color":[0.9,0.08,0.06],"trail":[0.9,0.08,0.06]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float tide = smoothstep(0.3, 1.0, sin(b.y*0.2 - uTime*0.9));
  float heart = 1.0 - 0.45*exp(-pow(b.x/0.25, 2.0));
  return g*heart*(0.8 + 0.5*tide);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 red = vec3(1.0, 0.08, 0.05), dark = vec3(0.55, 0.0, 0.02);
  fx += fxFall(ro, rd, px, y0, y1, 1.4, 12.0, 5.0, 0.6, 0.0, 2.9, red)*part(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.6, 1.8, 10.0, -0.05, 4.4, red)*0.8*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.6, 2.0, 5.3, dark)*1.2*part(1.0);
  // round 16 (Creyon liked the earlier look): its two dark red streams back
  fx += fxStreams(ro, rd, px, L*0.04, y1, 2.0, 3.2, 0.18, 1.2, 0.7, dark*1.4)*part(2.0);
  return fx*uFx;
}
// round 16 (Creyon: Sol's blades' pieces are 3D): the blood moon circles the blade slowly, rising
// and sinking, a red ball lit from one side with its shadow sliding across, a red halo round it;
// drawn in true 3D round the blade (Sol's way)
vec3 bm_moon(){ float L = uLength; return around(3.0, uTime*0.07, L*(0.68 + 0.12*sin(uTime*0.3))); }
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5) { return vec3(0.0); }
  float ta;
  float d = rayPt(ro, rd, bm_moon(), ta);
  return (vec3(1.0, 0.12, 0.06)*halo(d - 1.35, 0.6)*0.6 + vec3(0.55, 0.0, 0.02)*halo(d, 3.0)*0.25)*uFx;
}
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  if (part(2.0) < 0.5 || uFx < 0.01) return vec4(0.0);
  float ta;
  vec3 c = bm_moon();
  float d = rayPt(ro, rd, c, ta);
  float R = 1.35;
  float cov = fillD(d - R, px);
  if (cov <= 0.0) return vec4(0.0);
  vec3 q = ro + rd*ta - c;
  float back = sqrt(max(R*R - d*d, 0.0));
  vec3 n = normalize(q - rd*back);
  vec3 sun = normalize(vec3(cos(uTime*0.05), 0.3, sin(uTime*0.05)));
  float lit = smoothstep(-0.15, 0.35, dot(n, sun));
  float mott = 0.75 + 0.25*vn(n.xy*3.0 + n.z*2.0 + 5.0);
  vec3 col = mix(vec3(0.35, 0.02, 0.01), vec3(1.0, 0.2, 0.1)*1.5, lit)*mott;
  // round 17: hidden behind the blade
  cov *= bladeSeen(ro, rd, c);
  uOwn = max(uOwn, cov);
  return vec4(col, cov);
}
