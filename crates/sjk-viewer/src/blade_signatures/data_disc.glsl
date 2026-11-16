// Data Disc: spinning data discs as a blade. Three discs of fine glowing grooves turn round it at
// different heights, the middle one the other way, each with a bright read-light sweeping round it
// and a glowing hub; packets of data stream up the blade between them and soft bands run up it
// (round 15, Creyon: the single disc looked cheap). A clash sends an amber ring.
// @part Data discs
// @part Data stream
// @room 8
// @palette {"corona.rim_cool":[0.5,0.18,0.04],"corona.rim_hot":[1.0,0.55,0.18],"corona.inner":[1.0,0.85,0.6],"core.white":[1.0,0.95,0.85],"core.fringe_cool":[1.0,0.5,0.15],"core.fringe_hot":[1.0,0.85,0.6],"light.color":[1.0,0.5,0.18],"trail":[1.0,0.5,0.18]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g*(0.9 + 0.12*bands(b.y, 0.1, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(1.0, 0.6, 0.25);
  vec3 disc = vec3(0.0);
  for (int k = 0; k < 3; k++){
    float fk = float(k);
    float yd = L*(0.38 + 0.24*fk);
    float R = 2.6 - 0.35*abs(fk - 1.0);
    float dir = k == 1 ? -1.0 : 1.0;
    vec3 h = rayAt(ro, rd, yd);
    if (h.z > 0.0){
      // round 23: hidden behind the blade softly (see hideD in lab.glsl)
      float r = length(h.xy), seen = 1.0 - behindBlade(ro, rd, h.z);
      float ang = atan(h.y, h.x);
      float on = smoothstep(0.75, 0.95, r)*(1.0 - smoothstep(R - 0.25, R, r));
      float gr = 0.5 + 0.5*cos(r*25.0 + fk);
      // sectors of written data, some bright, turning with the disc
      float sec = step(0.45, h11(floor((ang - t*1.5*dir)*6.0/3.1416) + fk*7.0));
      float sweep = pow(0.5 + 0.5*cos(ang - t*1.5*dir + fk*2.0), 10.0);
      disc += c*on*(0.12 + 0.14*gr + 0.12*sec + 0.9*sweep*gr)*seen;
      // the hub
      disc += vec3(1.0, 0.85, 0.6)*halo(r - 0.75, 0.08)*0.8*seen;
    }
    disc += c*halo(ringD(ro, rd, yd, R), max(0.04, px))*0.6;
  }
  fx += disc*part(0.0);
  // data packets: small blocks of light streaming up the blade between the discs
  float s;
  if (fxNear(ro, rd, y0, y1) < 2.5){
    for (int i = 0; i < 14; i++){
      float fi = float(i);
      float q = fract(t*(0.35 + 0.2*h11(fi*2.7)) + h11(fi*6.1));
      float y = mix(y0, y1, q);
      float side = mod(fi, 2.0) < 0.5 ? -1.0 : 1.0;
      float x = side*(0.55 + 0.35*h11(fi*3.9));
      // round 19: all round the blade, in three dimensions
      float d = raySeg(ro, rd, turnY(vec3(x, y - 0.25, 0.0), h11(fi*8.3)*0.5), turnY(vec3(x, y + 0.25, 0.0), h11(fi*8.3)*0.5), s);
      float bit = step(0.35, h11(floor(t*8.0) + fi*3.0));
      fx += (vec3(1.0, 0.9, 0.7)*halo(d, max(0.04, px))*1.2 + c*halo(d, 0.15)*0.5)*(0.4 + 0.6*bit)*sin(3.1416*q)*part(1.0);
    }
  }
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx;
  return vec3(0.0);
}
