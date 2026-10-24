// Holonet Feed: the HoloNet's endless news feed, as a blade. Rows of data dashes scroll up through
// the blade, brighter and dimmer, and small flecks of data drift up beside it. No letters or signs
// are drawn.
// @part Data feed
// @part Data flecks
// @motes Data flecks
// @room 8
// @palette {"corona.rim_cool":[0.04,0.4,0.16],"corona.rim_hot":[0.3,1.0,0.5],"corona.inner":[0.7,1.0,0.8],"core.white":[0.9,1.0,0.94],"core.fringe_cool":[0.2,0.95,0.45],"core.fringe_hot":[0.75,1.0,0.85],"light.color":[0.25,1.0,0.5],"trail":[0.3,1.0,0.5]}

vec3 holonet_feed_glow0(vec3 g, vec2 b, float r, float px){
  // round 23: the dashes' ends and sides smoothed (the next dash began at a hard line)
  float u = b.y*1.5 - uTime*3.0;
  vec2 q = vec2(floor(b.x*6.0), floor(u));
  float lit = mix(step(0.5, h21(q)), step(0.5, h21(q + vec2(1.0, 0.0))), cellMix(b.x*6.0, px*6.0));
  float on = lit*smoothstep(0.1, 0.3, fract(u))*(1.0 - smoothstep(1.0 - min(px*1.5 + 0.05, 0.5), 1.0, fract(u)));
  return g*(0.75 + 0.45*on*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = holonet_feed_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.7, 2.5, 14.0, 0.12, 7.2, vec3(0.8, 0.9, 1.0))*0.6*part(1.0);
  return fx*uFx*2.40;
}
