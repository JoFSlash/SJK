// Imperial Schematics: the schematics of an Imperial battle station, as a blade. Bands of a technical drawing
// light up the blade as a drawing head climbs it, and two measuring lines with tick marks draw
// themselves up beside it, then fade and start again. A clash flares it.
// @part Schematic lines
// @room 8
// @palette {"corona.rim_cool":[0.05,0.2,0.6],"corona.rim_hot":[0.3,0.55,1.0],"corona.inner":[0.7,0.82,1.0],"core.white":[0.9,0.95,1.0],"core.fringe_cool":[0.25,0.45,1.0],"core.fringe_hot":[0.7,0.85,1.0],"light.color":[0.25,0.45,1.0],"trail":[0.25,0.48,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float cyc = mod(uTime*6.0, uLength + 10.0);
  float head = min(cyc, uLength);
  float fade = 1.0 - smoothstep(uLength, uLength + 10.0, cyc);
  float line = halo(abs(b.y - 2.5*floor(b.y/2.5 + 0.5)), 0.08)*step(b.y, head)*fade;
  return g*(0.85 + 0.6*line*part(0.0)) + lvl(g)*vec3(0.8, 0.9, 1.0)*halo(abs(b.y - head), 0.5)*0.5*part(0.0);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.75, 0.88, 1.0);
  float cyc = mod(t*6.0, L + 10.0);
  float head = min(cyc, y1);
  float fade = 1.0 - smoothstep(L, L + 10.0, cyc);
  float s;
  if (head > y0 && fxNear(ro, rd, y0, y1) < 3.5){
    // round 19: four rails round the blade, in three dimensions
    for (int k = 0; k < 4; k++){
      float sx = 1.8;
      float tk = float(k)*0.25 + 0.125;
      float d = raySeg(ro, rd, turnY(vec3(sx, y0, 0.0), tk), turnY(vec3(sx, head, 0.0), tk), s);
      fx += c*(halo(d, max(0.03, px))*0.9 + halo(d, 0.25)*0.1)*fade;
      float e = rayPt(ro, rd, turnY(vec3(sx, head, 0.0), tk), s);
      fx += vec3(1.0)*halo(e, 0.15)*1.2*fade;
    }
    for (int i = 0; i < 8; i++){
      float y = y0 + 4.0*float(i);
      if (y > head) break;
      // round 20: each rung a ring joining the four rails, in 3D
      float d = ringD(ro, rd, y, 1.8);
      fx += c*halo(d, max(0.03, px))*0.8*fade;
    }
  }
  return fx*uFx*part(0.0);
}
