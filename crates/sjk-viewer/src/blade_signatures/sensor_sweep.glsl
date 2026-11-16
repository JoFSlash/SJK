// Sensor Sweep: a starship's sensor screen as a blade. A bright sweep runs up the blade again and
// again, and small contacts light up beside it as it passes and fade away.
// @part Sweep
// @part Contacts
// @veins Sensor grid
// @room 8
// @palette {"corona.rim_cool":[0.05,0.35,0.1],"corona.rim_hot":[0.35,1.0,0.45],"corona.inner":[0.85,1.0,0.85],"core.white":[0.95,1,0.95],"core.fringe_cool":[0.3,1.0,0.4],"core.fringe_hot":[0.8,1.0,0.8],"light.color":[0.35,1.0,0.45],"trail":[0.35,1.0,0.45]}

vec3 sensor_sweep_glow0(vec3 g, vec2 b, float r, float px){
  float q = fract(uTime/3.0);
  float sweep = halo(b.y - q*uLength*1.1, 1.2) + 0.4*halo(b.y - q*uLength*1.1 + 2.5, 3.0);
  return g*(0.75 + 0.8*sweep*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = sensor_sweep_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  float ta;
  float k = floor(t/3.0), q = fract(t/3.0);
  for (int i = 0; i < 6; i++){
    float fi = float(i);
    float yb = mix(y0, y1, h11(k*3.3 + fi*1.7));
    float age = q - yb/(L*1.1);
    if (age < 0.0) continue;
    vec3 p = vec3((h11(k*1.9 + fi) < 0.5 ? -1.0 : 1.0)*(1.0 + 1.8*h11(k*5.1 + fi)), yb, 0.0);
    float d = rayPt(ro, rd, p, ta);
    fx += vec3(0.6, 1.0, 0.65)*(halo(d, max(0.07, px))*1.4 + halo(d, 0.35)*0.3)*exp(-age*3.0)*part(1.0);
  }
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*2.40;
  return vec3(0.0);
}
