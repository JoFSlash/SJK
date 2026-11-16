// Otoh Gunga: the Gungans' bubble city under the lakes of Naboo, as a blade. Lake light shimmers
// through the blade in soft moving patterns and wobbling bubbles drift up round it.
// @part Bubbles
// @part Water shimmer
// @wisps Lake haze
// @room 8
// @palette {"corona.rim_cool":[0.05,0.3,0.45],"corona.rim_hot":[0.35,0.85,1.0],"corona.inner":[0.85,0.97,1.0],"core.white":[0.95,1,1],"core.fringe_cool":[0.3,0.8,1.0],"core.fringe_hot":[0.8,0.95,1.0],"light.color":[0.35,0.85,1.0],"trail":[0.35,0.85,1.0]}

vec3 otoh_gunga_glow0(vec3 g, vec2 b, float r, float px){
  float sh = pow(0.5 + 0.5*sin(b.y*0.6 - uTime*1.2 + 2.0*sin(b.x*6.0 + uTime)), 3.0);
  return g*(0.85 + 0.3*sh*part(1.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = otoh_gunga_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.7, 0.95, 1.0);
  float ta;
  if (fxNear(ro, rd, y0, y1) < 3.0){
    for (int i = 0; i < 14; i++){
      float fi = float(i);
      float f = fract(t*0.09*(0.7 + 0.6*h11(fi*2.3)) + h11(fi*4.7));
      // round 20: all round the blade in 3D (they rose in one flat sheet)
      vec3 p = around(0.7 + 1.5*h11(fi*3.1) + 0.2*sin(t*2.0 + fi*1.3), h11(fi*6.9), mix(y0, y1, f));
      float r = 0.18 + 0.25*h11(fi*1.7);
      float d = rayPt(ro, rd, p, ta);
      fx += (c*halo(abs(d - r), max(0.03, px))*0.9 + c*halo(d, r)*0.12)*sin(3.1416*f)*part(0.0);
    }
  }
  return fx*uFx*2.40;
}
