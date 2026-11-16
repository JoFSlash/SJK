// Cantina Lights: a night in a busy spaceport cantina, as a blade. The blade cycles slowly through the
// stage lights' colours, soft coloured spots of light sweep back and forth beside it and a thin
// smoky haze hangs round it.
// @part Cantina lights
// @part Smoky air
// @wisps Cantina smoke
// @room 8
// @palette {"corona.rim_cool":[0.2,0.1,0.45],"corona.rim_hot":[0.6,0.4,1.0],"corona.inner":[0.9,0.85,1.0],"core.white":[0.95,0.93,1.0],"core.fringe_cool":[0.5,0.35,1.0],"core.fringe_hot":[0.85,0.8,1.0],"light.color":[0.6,0.4,1.0],"trail":[0.6,0.4,1.0]}

vec3 cantina_lights_glow0(vec3 g, vec2 b, float r, float px){
  vec3 c = hsv(fract(uTime*0.05 + b.y*0.015), 0.65, 1.0);
  return mix(g, recol(g, c), part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = cantina_lights_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  float ta;
  for (int i = 0; i < 5; i++){
    float fi = float(i);
    // round 20: the spots of light circle the blade in 3D (they swung in one flat sheet)
    float a = t*(0.4 + 0.15*fi) + fi*1.7;
    vec3 p = vec3(2.6*sin(a), mix(y0 + 2.0, y1 - 2.0, (fi + 0.5)/5.0), 2.6*cos(a));
    float d = rayPt(ro, rd, p, ta);
    fx += hsv(fract(fi*0.21 + t*0.05), 0.7, 1.0)*(halo(d, 0.5)*0.45 + halo(d, 1.4)*0.12)*part(0.0);
  }
  fx += fxSmoke(ro, rd, y0, y1, 2.8, 0.8, 1.2, vec3(0.45, 0.35, 0.55))*0.5*part(1.0);
  return fx*uFx*2.40;
}
