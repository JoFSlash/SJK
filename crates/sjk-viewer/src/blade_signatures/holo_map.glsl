// Holo-Map: a tactical hologram on a briefing table, as a blade. The blade is drawn in fine
// wireframe lines along and across it that slowly turn, flickering now and then like a projection,
// with a faint holo glow round it.
// @part Wireframe
// @part Holo glow
// @echo Projection echo
// @room 8
// @palette {"corona.rim_cool":[0.05,0.3,0.4],"corona.rim_hot":[0.35,0.9,1.0],"corona.inner":[0.8,0.97,1.0],"core.white":[0.92,1,1],"core.fringe_cool":[0.3,0.85,1.0],"core.fringe_hot":[0.75,0.95,1.0],"light.color":[0.35,0.9,1.0],"trail":[0.35,0.9,1.0]}

vec3 holo_map_glow0(vec3 g, vec2 b, float r, float px){
  float turn = uTime*0.6;
  float along = halo(abs(fract(b.x*3.0 + turn) - 0.5) - 0.0, 0.06);
  float across = halo((0.5 - abs(fract(b.y/2.0) - 0.5))*2.0, 0.04);
  float fl = 0.85 + 0.15*step(0.95, h11(floor(uTime*8.0)));
  float wire = clamp(along*0.0 + across, 0.0, 1.0);
  float lines = halo(abs(abs(b.x) - 0.3*abs(cos(turn))), 0.04);
  return g*(0.55 + 0.25*fl) + lvl(g)*vec3(0.8, 1.0, 1.0)*clamp(wire + lines, 0.0, 1.0)*0.7*fl*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = holo_map_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 2.0, vec3(0.35, 0.9, 1.0))*0.3*part(1.0);
  return fx*uFx*2.40;
}
