// Orto Plutonia: the frozen world of Orto Plutonia, as a blade. Snow swirls round the blade in slow
// circles and settles, whirled up harder when it swings, and a frost tints its light.
// @part Swirling snow
// @part Frost
// @wisps Blowing snow
// @room 8
// @palette {"corona.rim_cool":[0.1,0.2,0.45],"corona.rim_hot":[0.45,0.65,1.0],"corona.inner":[0.8,0.9,1.0],"core.white":[0.95,0.97,1.0],"core.fringe_cool":[0.4,0.6,1.0],"core.fringe_hot":[0.85,0.92,1.0],"light.color":[0.55,0.7,1.0],"trail":[0.6,0.75,1.0]}

vec3 orto_plutonia_glow0(vec3 g, vec2 b, float r, float px){
  return mix(g, recol(g, vec3(0.88, 0.95, 1.0))*1.05, 0.35*part(1.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = orto_plutonia_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 snow = vec3(0.92, 0.97, 1.0);
  float ta;
  // round 23: out to where the swirling snow's glow has faded (it swings out to 2.9 units and
  // was cut at 3)
  if (fxNear(ro, rd, y0, y1) < 4.5){
    for (int i = 0; i < 18; i++){
      float fi = float(i);
      float a = t*(0.25 + 0.2*h11(fi*2.1)) + h11(fi*5.3)*6.2832;
      float r = 0.7 + (1.2 + 1.0*sw)*h11(fi*3.7);
      float y = mix(y0, y1, fract(h11(fi*1.9) - t*0.02*(0.5 + h11(fi))));
      // round 19: circling the blade in three dimensions
      vec3 p = vec3(r*sin(a), y + 0.6*sin(a*2.0), r*cos(a));
      float d = rayPt(ro, rd, p, ta);
      fx += snow*(halo(d, max(0.06, px))*1.1 + halo(d, 0.25)*0.15)*part(0.0);
    }
  }
  return fx*uFx*2.40;
}
