// Cortosis Weave: a blade woven of cortosis threads, as a blade. A fine diagonal weave runs through
// its light and a soft glint slides along it now and then.
// @part Glint
// @veins Weave threads
// @room 8
// @palette {"corona.rim_cool":[0.08,0.25,0.65],"corona.rim_hot":[0.3,0.6,1.0],"corona.inner":[0.7,0.85,1.0],"core.white":[0.9,0.95,1.0],"core.fringe_cool":[0.25,0.5,1.0],"core.fringe_hot":[0.6,0.8,1.0],"light.color":[0.25,0.55,1.0],"trail":[0.28,0.58,1.0]}

vec3 cortosis_weave_glow0(vec3 g, vec2 b, float r, float px){
  float w = 0.5 + 0.5*sin((b.x*9.0 + b.y*1.2))*sin((b.x*9.0 - b.y*1.2));
  float glint = halo(b.y - mod(uTime*9.0, uLength + 20.0) + 10.0, 2.0);
  return g*(0.8 + 0.25*w) + lvl(g)*vec3(1.0)*glint*0.5*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = cortosis_weave_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += vec3(0.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*2.00;
  return vec3(0.0);
}
