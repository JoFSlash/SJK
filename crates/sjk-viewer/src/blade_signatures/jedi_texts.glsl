// Jedi Texts: the old sacred texts of the Jedi as a blade. Two flowing strokes of golden ink light
// curl up beside it, thick and thin like a brush's stroke, slowly rewriting themselves; there are
// no letters, only the flow of the writing. A clash flares it.
// @part Ink strokes
// @room 8
// @palette {"corona.rim_cool":[0.2,0.12,0.45],"corona.rim_hot":[0.95,0.75,0.4],"corona.inner":[1.0,0.92,0.75],"core.white":[1.0,0.96,0.88],"core.fringe_cool":[0.6,0.45,0.9],"core.fringe_hot":[1.0,0.88,0.65],"light.color":[0.95,0.72,0.38],"trail":[0.95,0.72,0.4]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(1.0, 0.82, 0.45);
  float s;
  if (fxNear(ro, rd, y0, y1) < 3.0){
    for (int k = 0; k < 2; k++){
      float fk = float(k);
      vec3 prev = vec3(0.0);
      float best = 1e3, bf = 0.0;
      for (int i = 0; i <= 16; i++){
        float f = float(i)/16.0;
        float y = mix(y0, y1, f);
        float x = (fk*2.0 - 1.0)*(0.9 + 0.5*sin(y*0.35 - t*0.9 + fk*2.0)) + 0.3*sin(y*1.1 + t*0.6);
        // round 19: the lines of script wind slowly round the blade, in three dimensions
        vec3 p = turnY(vec3(x, y, 0.0), y*0.012 + t*0.01);
        if (i > 0){
          float d = raySeg(ro, rd, prev, p, s);
          if (d < best){ best = d; bf = (float(i) - 1.0 + s)/16.0; }
        }
        prev = p;
      }
      float stroke = 0.5 + 0.5*sin(bf*14.0 - t*1.5 + fk*3.0);
      float w = 0.04 + 0.08*stroke;
      fx += (vec3(1.0)*halo(best, max(w*0.5, px))*0.8 + c*halo(best, w*2.5)*0.7)*(0.4 + 0.6*stroke)*smoothstep(0.0, 0.08, bf)*(1.0 - smoothstep(0.85, 1.0, bf));
    }
  }
  return fx*uFx*part(0.0);
}
