// Bacta: a blade of glowing bacta, the healing fluid of the galaxy's medical tanks. A blue-green
// blade with slow currents of brighter fluid rising through it; bubbles of every size stream up
// round it, wobbling and catching the light, and a soft healing glow pulses up it. A swing sends
// a rush of bubbles off it; a clash bursts a cloud of them.
// @part Bacta bubbles
// @room 6
// @palette {"corona.rim_cool":[0.02,0.35,0.35],"corona.rim_hot":[0.25,0.95,0.80],"corona.inner":[0.75,1.0,0.92],"core.white":[0.92,1.0,0.97],"core.fringe_cool":[0.20,0.85,0.70],"core.fringe_hot":[0.70,1.0,0.90],"light.color":[0.30,0.95,0.80],"trail":[0.35,0.95,0.80]}

float bt_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
vec3 base_vol(vec3 ro, vec3 rd, float px){
  float t = uTime, top = bt_top(), ta;
  float sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 tl = vec3(0.3, 0.95, 0.8), pale = vec3(0.85, 1.0, 0.95);
  vec3 col = vec3(0.0);
  if (top < 0.5) return col;
  // bubbles streaming up round the blade: each a ring of light facing the camera with a glint
  for (int i = 0; i < 24; i++){
    float fi = float(i);
    float rate = 0.18 + 0.15*h11(fi*2.3);
    float f = fract(t*rate + h11(fi*7.1));
    float k = floor(t*rate + h11(fi*7.1));
    float R = 0.08 + 0.28*h11(fi*3.9 + k);
    float y = -0.5 + f*(top + 3.0);
    if (y > top + 1.5) continue;
    vec3 c = around(0.7 + 2.2*h11(fi*5.1 + k) + 0.25*sin(t*3.0 + fi), h11(fi*1.7 + k) + 0.05*sin(t + fi), y);
    float tc;
    float d = rayPt(ro, rd, c, tc);
    vec3 off = ro + rd*tc - c;
    float rim = halo(abs(d - R), max(0.025, px*0.6));
    float glint = halo(length(off - normalize(vec3(-0.5, 0.7, 0.0))*R*0.55), R*0.25 + px);
    float fade = smoothstep(0.0, 0.08, f)*(1.0 - smoothstep(0.85, 1.0, f));
    col += (pale*rim*1.0 + vec3(1.0)*glint*0.8 + tl*halo(d, R)*0.08)*fade;
  }
  // a clash: a cloud of small bubbles bursting off the struck point
  if (cl > 0.01){
    for (int i = 0; i < 14; i++){
      float fi = float(i);
      vec3 dir = normalize(vec3(sin(fi*2.4), 0.6*(h11(fi*3.0) - 0.3), cos(fi*2.4)));
      vec3 c = vec3(0.0, uLength*0.6, 0.0) + dir*(0.6 + 4.0*(1.0 - cl));
      float d = rayPt(ro, rd, c, ta);
      col += pale*halo(abs(d - 0.15), max(0.025, px))*1.4*cl;
    }
  }
  return col*uFx;
}
// the blade: Sol's glow and core from the file, slow currents of brighter fluid rising in it and
// a healing glow pulsing up it
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float cur = 0.5 + 0.5*sin(b.y*0.6 - uTime*1.8 + 2.0*sin(b.x*3.0 + uTime*0.7));
  float pulse = pow(0.5 + 0.5*sin(b.y*0.15 - uTime*1.2), 6.0);
  return g*(0.85 + 0.25*cur + 0.5*pulse + 0.4*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// round 11: sig_vol gated by part 0
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  return base_vol(ro, rd, px);
}
