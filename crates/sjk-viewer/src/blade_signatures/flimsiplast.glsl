// Flimsiplast: folded sheets of flimsiplast, as a blade. Its light is creased into sharp facets,
// some brighter, some darker, that slowly refold into new ones, and small sheets of light drift
// down from it.
// @part Folds
// @part Drifting flimsi
// @motes Flecks of light
// @room 8
// @palette {"corona.rim_cool":[0.55,0.1,0.1],"corona.rim_hot":[1.0,0.45,0.35],"corona.inner":[1.0,0.9,0.85],"core.white":[1.0,0.97,0.92],"core.fringe_cool":[0.95,0.35,0.3],"core.fringe_hot":[1.0,0.85,0.8],"light.color":[1.0,0.5,0.4],"trail":[1.0,0.5,0.4]}

vec3 flimsiplast_glow0(vec3 g, vec2 b, float r, float px){
  float k = floor(uTime/4.0), q = smoothstep(0.0, 0.3, fract(uTime/4.0));
  // round 23: the creases' edges smoothed over a pixel (they stair-stepped)
  float m0 = 0.8 + 0.4*h11(k), m1 = 0.8 + 0.4*h11(k + 1.0);
  float s = mix(aaSquare(b.y/3.0 + b.x*m0, px*(0.34 + m0)), aaSquare(b.y/3.0 + b.x*m1, px*(0.34 + m1)), q);
  return g*(1.0 - 0.3*s*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = flimsiplast_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 2.6, 7.0, 1.6, 0.25, 0.9, 1.0, 0.6, 2.9, vec3(0.95, 0.95, 1.0))*0.7*part(1.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*2.40;
  return vec3(0.0);
}
