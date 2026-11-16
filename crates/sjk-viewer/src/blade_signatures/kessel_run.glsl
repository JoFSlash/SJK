// Kessel Run: a blade that flies like a ship on the run. An icy-blue blade with speed bands racing
// up it sheds two long tails of light, a blue one and a gold one, a blue exhaust rolls back off it
// and sparks peel away; afterimages trail every swing. A clash flares it.
// @part Comet tails
// @part Exhaust
// @part Sparks
// @room 8
// @palette {"corona.rim_cool":[0.2,0.45,0.9],"corona.rim_hot":[0.6,0.85,1.0],"corona.inner":[0.9,0.97,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.4,0.7,1.0],"core.fringe_hot":[0.85,0.95,1.0],"light.color":[0.55,0.8,1.0],"trail":[0.55,0.8,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float race = pow(0.5 + 0.5*sin(b.y*0.6 - uTime*14.0), 6.0);
  return g*(0.85 + 0.45*race);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 blue = vec3(0.5, 0.75, 1.0), gold = vec3(1.0, 0.8, 0.4);
  fx += fxFall(ro, rd, px, y0, y1, 2.5, 14.0, 14.0, 1.6, -0.6, 3.3, blue)*part(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 2.0, 10.0, 11.0, 1.2, 0.6, 7.1, gold)*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.2, -4.0, 2.2, blue*0.8)*part(1.0);
  fx += fxSparks(ro, rd, px, y0, y1, 8.0 + 10.0*sw, 4.5, 0.8, 5.5, vec3(0.85, 0.93, 1.0))*part(2.0);
  return fx*uFx;
}
