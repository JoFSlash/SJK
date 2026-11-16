// Takodana Rain: soft rain on the lake below Takodana's castle, as a blade. Drops land on the blade
// and send ripples of light running along it both ways, and a light rain falls past it. Round 20
// (Creyon: it looked like Starfighter Canopy): straight soft rain, and where each drop lands a
// ring ripples out round the blade in 3D as on the lake's face.
// @part Ripples
// @part Rain
// @wisps Lake mist
// @room 8
// @palette {"corona.rim_cool":[0.15,0.3,0.5],"corona.rim_hot":[0.55,0.8,1.0],"corona.inner":[0.88,0.95,1.0],"core.white":[0.95,0.98,1.0],"core.fringe_cool":[0.45,0.7,1.0],"core.fringe_hot":[0.85,0.93,1.0],"light.color":[0.55,0.8,1.0],"trail":[0.58,0.82,1.0]}

vec3 takodana_rain_glow0(vec3 g, vec2 b, float r, float px){
  float rip = 0.0;
  for (int i = 0; i < 3; i++){
    float fi = float(i);
    float cyc = uTime*0.5 + fi*0.33;
    float k = floor(cyc), age = fract(cyc)*2.0;
    float yd = uLength*(0.25 + 0.65*h11(k*2.3 + fi));
    float d = abs(b.y - yd);
    rip += max(sin(d*3.0 - age*8.0), 0.0)*exp(-d*0.35)*exp(-age*1.5);
  }
  return g*(0.85 + 0.6*rip*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = takodana_rain_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 3.0, 10.0, 10.0, 0.5, 0.0, 5.7, vec3(0.7, 0.9, 0.95))*0.45*part(1.0);
  // the drops landing: a ring rippling out round the blade where each lands, as on the lake
  for (int i = 0; i < 3; i++){
    float fi = float(i);
    float cyc = t*0.5 + fi*0.33;
    float k = floor(cyc), age = fract(cyc)*2.0;
    float yd = L*(0.25 + 0.65*h11(k*2.3 + fi));
    float env = exp(-age*1.6)*smoothstep(0.0, 0.08, age);
    for (int j = 0; j < 2; j++){
      float R = 0.6 + 2.2*max(age - float(j)*0.25, 0.0);
      float d = ringD(ro, rd, yd, R);
      fx += vec3(0.7, 0.95, 1.0)*(halo(d, max(0.04, px))*0.9 + halo(d, 0.25)*0.2)*env*(1.0 - 0.4*float(j))*part(0.0);
    }
  }
  return fx*uFx*2.40;
}
