// Tipoca Stormglass: an old weather glass from Tipoca City on stormy Kamino as a blade. Feathery crystals slowly
// form and dissolve inside a sea-green blade, and thin flickers of storm lightning cross it now and
// then. A clash flares it.
// @part Storm crystals
// @arcs Storm flickers
// @room 8
// @palette {"corona.rim_cool":[0.22,0.36,0.38],"corona.rim_hot":[0.62,0.82,0.8],"corona.inner":[0.85,0.96,0.94],"core.white":[0.95,0.99,0.98],"core.fringe_cool":[0.55,0.78,0.76],"core.fringe_hot":[0.85,0.96,0.95],"light.color":[0.66,0.88,0.86],"trail":[0.62,0.86,0.84]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float n = fbm3(vec2(b.x*3.0 + 0.5*sin(b.y*0.3), b.y*0.6 + uTime*0.05));
  float form = 0.5 + 0.5*sin(uTime*0.25 + b.y*0.05);
  float cr = smoothstep(0.62 - 0.12*form, 0.66 - 0.12*form, n)*part(0.0);
  return g*(0.8 + 0.2*form) + lvl(g)*vec3(0.85, 1.0, 0.98)*cr*0.9;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 1.6, vec3(0.6, 0.9, 0.88))*0.25;
  return fx*uFx*part(0.0);
}
