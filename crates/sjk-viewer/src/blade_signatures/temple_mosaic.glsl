// Temple Mosaic: an old Jedi Temple's tiled floor as a blade. Small tiles of gold, teal and red
// with dark joints between them, each catching the light on its own now and then, and a warm glow
// round the blade. A clash flares it.
// @part Tile light
// @room 8
// @palette {"corona.rim_cool":[0.55,0.3,0.08],"corona.rim_hot":[1.0,0.72,0.3],"corona.inner":[1.0,0.92,0.72],"core.white":[1.0,0.97,0.9],"core.fringe_cool":[1.0,0.65,0.25],"core.fringe_hot":[1.0,0.9,0.65],"light.color":[1.0,0.7,0.3],"trail":[1.0,0.7,0.3]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec2 q = vec2(b.x*3.0, b.y*0.8);
  vec2 id = floor(q), f = fract(q);
  float h = h21(id);
  vec3 tile = h < 0.33 ? vec3(1.0, 0.75, 0.35) : (h < 0.66 ? vec3(0.35, 0.8, 0.85) : vec3(0.95, 0.4, 0.3));
  float grout = smoothstep(0.0, 0.12, min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y)));
  float glint = pow(0.5 + 0.5*sin(uTime*1.5 + h*40.0), 16.0);
  return recol(g, tile)*(0.55 + 0.45*grout)*(1.0 + 1.2*glint*part(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 warm = vec3(1.0, 0.75, 0.4);
  fx += fxAura(ro, rd, y0, y1, 2.0, warm)*0.5;
  return fx*uFx*part(0.0);
}
