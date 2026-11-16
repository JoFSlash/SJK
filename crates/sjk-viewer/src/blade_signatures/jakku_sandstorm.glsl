// Jakku Sandstorm: a sandstorm over the Jakku wastes as a blade. Streaks of sand blow slowly across
// it from side to side, a haze of dust rolls off it and the hot air shimmers; round 16 (Creyon:
// better for its name and rarity): a dust devil whirls round it in 3D, sand swept up in a funnel,
// and blown grit scours the blade's light. A clash flares it.
// @part Blowing sand
// @part Dust devil
// @room 8
// @palette {"corona.rim_cool":[0.55,0.35,0.12],"corona.rim_hot":[0.95,0.75,0.4],"corona.inner":[1.0,0.92,0.75],"core.white":[1.0,0.98,0.92],"core.fringe_cool":[0.95,0.72,0.35],"core.fringe_hot":[1.0,0.92,0.75],"light.color":[0.95,0.72,0.38],"trail":[0.92,0.7,0.38]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  // grit scouring it sideways: fine streaks dimming and brightening the light
  float grit = vn(vec2(b.x*2.0 - uTime*6.0, b.y*3.0));
  return g*(0.85 + 0.12*bands(b.y, 0.05, 0.2) + 0.25*(grit - 0.5));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 sand = vec3(0.95, 0.75, 0.45);
  float s;
  if (fxNear(ro, rd, y0, y1) < 6.0){
    for (int i = 0; i < 24; i++){
      float fi = float(i);
      float f = fract(t*0.18*(0.7 + 0.6*h11(fi*2.3)) + h11(fi*5.9));
      float y = mix(y0, y1, h11(fi*1.7)) + 1.5*sin(f*6.2832 + fi);
      float x = mix(-5.0, 5.0, f);
      // round 19: each streak at its own depth, so the storm blows past in three dimensions
      float z = (h11(fi*4.7) - 0.5)*7.0;
      vec3 a = vec3(x, y, z);
      vec3 e = vec3(x + 0.8 + 0.6*h11(fi*3.3), y + 0.1, z + 0.2);
      float d = raySeg(ro, rd, a, e, s);
      fx += sand*(halo(d, max(0.04, px))*0.6 + halo(d, 0.25)*0.12)*sin(3.1416*f);
    }
  }
  fx += fxSmoke(ro, rd, y0, y1, 3.6, 1.2, 4.1, vec3(0.6, 0.45, 0.25))*0.6;
  return fx*uFx*part(0.0);
}

// round 16: a dust devil whirling round the blade, sand swept up a widening funnel; it circles the
// blade, so it is drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(1.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, ta;
  float y0 = L*0.1, y1 = L*0.95;
  vec3 fx = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) > 5.0) return fx;
  vec3 sand = vec3(0.95, 0.75, 0.45);
  for (int i = 0; i < 36; i++){
    float fi = float(i);
    float f = fract(t*(0.08 + 0.05*h11(fi*2.9)) + h11(fi*5.1));
    float y = mix(y0, y1, f);
    float R = 0.8 + 3.0*f*f + 0.3*h11(fi*3.3);
    // round 19 (Creyon: the dust devil went far too fast): about a third as fast
    vec3 p = around(R, h11(fi*1.7) + t*(0.2 - 0.09*f), y);
    vec3 p0 = around(R, h11(fi*1.7) + (t - 0.22)*(0.2 - 0.09*f), y);
    float s;
    float d = raySeg(ro, rd, p0, p, s);
    fx += sand*(halo(d, max(0.035, px))*1.1 + halo(d, 0.18)*0.12)*sin(3.1416*f);
  }
  return fx*uFx;
}
