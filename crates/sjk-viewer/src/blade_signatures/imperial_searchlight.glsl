// Imperial Searchlight: a searchlight sweeping an Imperial base at night, as a blade. A bright band
// of light sweeps slowly up and down the blade and across it, and a cold night haze hangs round it.
// @part Searchlight
// @part Night haze
// @motes Dust in the beam
// @room 8
// @palette {"corona.rim_cool":[0.75,0.55,0.2],"corona.rim_hot":[1.0,0.92,0.65],"corona.inner":[1.0,0.97,0.85],"core.white":[1.0,0.98,0.9],"core.fringe_cool":[1.0,0.8,0.45],"core.fringe_hot":[1.0,0.95,0.8],"light.color":[1.0,0.9,0.6],"trail":[1.0,0.9,0.6]}

vec3 imperial_searchlight_glow0(vec3 g, vec2 b, float r, float px){
  float band = halo(b.y - (0.5 + 0.42*sin(uTime*0.7))*uLength, 2.5);
  float across = 0.6 + 0.4*halo(b.x - 0.3*sin(uTime*1.9), 0.25);
  return g*(0.75 + 1.0*band*across*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = imperial_searchlight_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.6, 0.6, 5.1, vec3(0.5, 0.6, 0.75))*0.55*part(1.0);
  return fx*uFx*2.40;
}
