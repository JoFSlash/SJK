// Moisture Farm: the dew lines of a Tatooine moisture farm as a blade. Four fine strands run from
// the hilt to the tip beside it, drops of dew slide slowly down them all the way to the hilt,
// glinting now and then, and a faint mist condenses round it in the dry heat. A clash flares it.
// @part Dew strands
// @part Condensing mist
// @room 8
// @palette {"corona.rim_cool":[0.2,0.26,0.48],"corona.rim_hot":[0.55,0.66,0.95],"corona.inner":[0.8,0.86,1.0],"core.white":[0.94,0.96,1.0],"core.fringe_cool":[0.5,0.6,0.92],"core.fringe_hot":[0.82,0.88,1.0],"light.color":[0.62,0.72,1.0],"trail":[0.62,0.72,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.75, 0.85, 1.0);
  float s, ta;
  float yb = 0.6;
  float ym = mix(yb, y1, 0.5);
  if (fxNear(ro, rd, yb, y1) < 3.0){
    for (int k = 0; k < 4; k++){
      float fk = float(k);
      float x = (k < 2 ? -1.0 : 1.0)*((k == 0 || k == 3) ? 2.0 : 1.2);
      // round 19: the four rods stand round the blade, in three dimensions
      float tk = fk*0.25 + 0.125;
      x = abs(x);
      vec3 a = turnY(vec3(x*0.45, yb, 0.0), tk), m = turnY(vec3(x*1.12, ym, 0.0), tk), e = turnY(vec3(x, y1 - 1.0, 0.0), tk);
      float d = min(raySeg(ro, rd, a, m, s), raySeg(ro, rd, m, e, s));
      fx += c*halo(d, max(0.015, px))*0.5*part(0.0);
      for (int i = 0; i < 4; i++){
        float fi = float(i);
        float f = fract(t*0.05*(0.8 + 0.4*h11(fk*3.0 + fi)) + h11(fk*7.0 + fi*2.3));
        float y = mix(y1 - 1.0, yb, f);
        vec3 pd = y > ym ? mix(m, e, (y - ym)/(y1 - 1.0 - ym)) : mix(a, m, (y - yb)/(ym - yb));
        float dd = rayPt(ro, rd, pd, ta);
        float tw = pow(0.5 + 0.5*sin(t*2.0 + fi*3.0 + fk), 8.0);
        fx += (vec3(1.0)*halo(dd, max(0.07, px))*(1.0 + 1.5*tw) + c*halo(dd, 0.3)*0.25)*smoothstep(0.0, 0.1, f)*(1.0 - smoothstep(0.92, 1.0, f))*part(0.0);
      }
    }
  }
  fx += fxSmoke(ro, rd, y0, y1, 2.6, 0.6, 3.9, vec3(0.6, 0.7, 0.85))*0.35*part(1.0);
  return fx*uFx;
}
