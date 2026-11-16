// Ion Coil: the coil of an ion cannon as a blade. Seven rings of the coil sit round it and pulse
// in turn from the hilt up, and short ion arcs jump off it with their own quick crackle. A clash flares it.
// @part Coil rings
// @arcs Ion arcs
// @room 8
// @palette {"corona.rim_cool":[0.25,0.15,0.65],"corona.rim_hot":[0.6,0.5,1.0],"corona.inner":[0.85,0.82,1.0],"core.white":[0.96,0.95,1.0],"core.fringe_cool":[0.55,0.45,1.0],"core.fringe_hot":[0.88,0.85,1.0],"light.color":[0.6,0.5,1.0],"trail":[0.6,0.5,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.65, 0.6, 1.0);
  for (int i = 0; i < 7; i++){
    float fi = float(i);
    float y = mix(y0, y1 - 2.0, fi/6.0);
    float pulse = pow(0.5 + 0.5*sin(t*3.0 - fi*0.9), 6.0);
    float d = ringD(ro, rd, y, 0.9);
    fx += vec3(1.0)*halo(d, max(0.04, px))*(0.5 + 1.2*pulse) + c*halo(d, 0.3)*(0.3 + 0.8*pulse);
  }
  return fx*uFx*part(0.0);
}
