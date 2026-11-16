// Senate Glass: the tall coloured windows of the old Galactic Senate, as a blade. Round 20 (Creyon:
// the effect was hardly visible): the blade itself is a column of stained glass, tall panes of deep
// red, blue and gold set in dark leading and lit from within, light slowly moving up through them
// as if the sun were passing outside, and coloured light from the panes falling round it.
// @part Light through the glass
// @motes Dust in the light
// @room 8
// @solid 1
// @palette {"corona.rim_cool":[0.45,0.2,0.4],"corona.rim_hot":[1.0,0.7,0.45],"corona.inner":[1.0,0.92,0.8],"core.white":[1.0,0.97,0.9],"core.fringe_cool":[0.8,0.35,0.5],"core.fringe_hot":[1.0,0.85,0.7],"light.color":[1.0,0.55,0.45],"trail":[0.95,0.5,0.55]}

// the pane at b (across, along): its colour, and the leading (0 on the lead, 1 in the glass)
vec3 sg_pane(vec2 b, out float lead){
  vec2 q = vec2(b.x*2.2 + 0.5, b.y/4.0);
  vec2 id = floor(q), f = fract(q);
  float h = h21(id);
  lead = smoothstep(0.0, 0.1, min(min(f.x, 1.0 - f.x), min(f.y, 1.0 - f.y)*2.5));
  return h < 0.33 ? vec3(0.95, 0.15, 0.15) : (h < 0.66 ? vec3(0.2, 0.4, 1.0) : vec3(1.0, 0.75, 0.2));
}
// round 20 (Creyon: let each pane light up as the light goes up): a wave of sunlight rising up the
// column, each row of panes brightening in turn as it passes, then settling back
float sg_sun(float y){
  float row = (floor(y/4.0) + 0.5)*4.0;
  float wave = fract(uTime*0.14)*(uLength + 12.0) - 6.0;
  return 0.6 + 1.3*exp(-pow((row - wave)/3.0, 2.0))*part(0.0);
}
// the glow round it takes the panes' colours
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float lead;
  vec3 pane = sg_pane(vec2(0.0, b.y), lead);
  return recol(g, mix(vec3(1.0, 0.9, 0.85), pane, 0.7))*sg_sun(b.y)*0.85;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*0.55; }
// the blade itself: the stained-glass column, lit from within, its leading dark
vec4 sig_solid(vec4 o, vec2 b, float px){
  float L = uLength;
  float w = 0.62*smoothstep(-0.3, 0.4, b.y)*(1.0 - smoothstep(L - 0.8, L + 0.1, b.y));
  float cov = fillD(abs(b.x) - w, px)*step(-0.3, b.y);
  if (cov <= 0.0) return o;
  float lead;
  vec3 pane = sg_pane(vec2(b.x/max(w, 0.01)*0.45, b.y), lead);
  float n = clamp(b.x/max(w, 0.01), -1.0, 1.0);
  float inner = 0.55 + 0.45*sqrt(max(1.0 - n*n, 0.0));
  float sun = sg_sun(b.y);
  vec3 glass = pane*(0.55 + 0.75*sun)*inner + vec3(1.0, 0.95, 0.85)*pow(max(1.0 - abs(n)*1.6, 0.0), 3.0)*0.6*sun;
  vec3 rgb = mix(vec3(0.05, 0.04, 0.05), glass, lead);
  return vec4(mix(o.rgb, rgb, cov), max(o.a, cov*0.92));
}
// coloured light from the panes falling round it, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength;
  float y0 = L*0.08, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  for (int k = 0; k < 3; k++){
    float fk = float(k);
    vec3 c = fk < 0.5 ? vec3(0.95, 0.2, 0.2) : (fk < 1.5 ? vec3(0.25, 0.45, 1.0) : vec3(1.0, 0.8, 0.3));
    float yc = mix(y0, y1, fract(0.2 + 0.31*fk + uTime*0.02));
    fx += fxAura(ro, rd, yc - 4.0, yc + 4.0, 1.8, c)*0.35*part(0.0);
  }
  return fx*uFx;
}
