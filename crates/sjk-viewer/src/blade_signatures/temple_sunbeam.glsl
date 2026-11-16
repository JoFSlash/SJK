// Temple Sunbeam: sunlight falling through a high Jedi Temple window, as a blade. A warm shaft of
// light round the blade, gently brighter in slow waves; soft slanted shafts of sunlight drift
// slowly across it, fine dust floats through the light and now and then a mote catches the sun
// and glints. Nothing reaches past the tip. A clash flares it.
// @part Dust in the light
// @part Sun shafts
// @part Glints
// @room 8
// @palette {"corona.rim_cool":[0.6,0.4,0.12],"corona.rim_hot":[1.0,0.85,0.5],"corona.inner":[1.0,0.96,0.82],"core.white":[1.0,0.99,0.93],"core.fringe_cool":[1.0,0.82,0.45],"core.fringe_hot":[1.0,0.95,0.8],"light.color":[1.0,0.85,0.5],"trail":[1.0,0.84,0.5]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g*(1.0 + 0.12*bands(b.y, 0.03, 0.2));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 warm = vec3(1.0, 0.88, 0.6);
  float yt = L*0.86;
  fx += fxAura(ro, rd, y0, yt, 3.0, warm)*0.45;
  fx += fxMotes(ro, rd, px, y0, yt - 2.0, 0.6, 3.0, 30.0, 0.012, 2.9, warm)*0.7*part(0.0);
  // soft slanted shafts of sunlight drifting slowly across the blade; round 19 (Creyon: from
  // above they were a flat picture following the camera): slabs of light in three dimensions
  // falling from one side, the ray marched through them
  {
    vec2 span = raySpan(ro, rd, 7.5, y0 - 2.0, yt);
    if (span.x < span.y){
      float dt = (span.y - span.x)/6.0, jit = 0.5, acc = 0.0;
      vec2 sd = vec2(0.82, 0.57);
      for (int i = 0; i < 6; i++){
        vec3 p = ro + rd*(span.x + (float(i) + jit)*dt);
        float xs = dot(p.xz, sd);
        float sh = pow(0.5 + 0.5*sin((xs*0.55 + p.y*0.28)*1.3 - t*0.35), 6.0)
                 + 0.6*pow(0.5 + 0.5*sin((xs*0.5 + p.y*0.25)*0.7 + t*0.2 + 1.7), 8.0);
        float win = exp(-pow(length(p.xz)/3.2, 2.0))*smoothstep(y0 - 2.0, y0 + 3.0, p.y)*(1.0 - smoothstep(yt - 4.0, yt, p.y));
        acc += sh*win;
      }
      fx += warm*acc*dt/(1.77*3.2)*0.55*part(1.0);
    }
  }
  // motes catching the sun: a few bright glints
  fx += fxMotes(ro, rd, px, y0, yt - 2.0, 0.8, 3.2, 8.0, 0.01, 6.4, vec3(1.0, 0.95, 0.8))*pow(0.5 + 0.5*sin(t*1.7), 4.0)*2.2*part(2.0);
  return fx*uFx;
}
