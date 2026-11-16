// Dac Reef: the coral reefs of Dac, the Mon Calamari world, as a blade (round 17 rework, Creyon: no
// small figures and no simple corals: the blade itself is one coral). The blade is a living branch
// of coral, a porous body of rose and sea-green stone grown round the light, its openings glowing
// with the blade shining through and polyps breathing in their rims; slow water light ripples
// over it, a soft sea haze drifts round it and the reef's glow pulses gently. A clash sends a ring
// through the water.
// @part Living coral
// @part Reef glow
// @room 8
// @solid 1
// @palette {"corona.rim_cool":[0.0,0.35,0.45],"corona.rim_hot":[0.3,0.95,0.95],"corona.inner":[0.8,1.0,1.0],"core.white":[0.92,1.0,1.0],"core.fringe_cool":[0.25,0.85,0.9],"core.fringe_hot":[0.78,1.0,1.0],"light.color":[0.3,0.9,0.95],"trail":[0.3,0.9,0.95]}

const vec3 DR_CORAL = vec3(0.85, 0.42, 0.45);
const vec3 DR_SEA = vec3(0.25, 0.85, 0.8);
float dr_top(){ return uLength*clamp(uAge/0.45, 0.0, 1.0); }
// the coral's width at a height: a little thicker than the blade, knobbly, rounded at its end
float dr_w(float y){
  float top = dr_top();
  return (0.75 + 0.15*vn(vec2(y*0.7, 3.0)))*smoothstep(-0.4, 0.6, y)*sqrt(clamp((top + 0.2 - y)/1.2, 0.0, 1.0));
}

// the light: water light rippling over it, the reef's glow pulsing gently
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float caust = pow(0.5 + 0.5*sin(b.y*0.8 - uTime*1.3 + 2.0*sin(b.x*6.0 + uTime*0.7)), 4.0);
  float pulse = 0.85 + 0.15*sin(uTime*0.9 - b.y*0.15)*part(1.0);
  return g*(0.85 + 0.35*caust)*pulse;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*0.9; }

// the coral itself: a porous stone body over the light, its openings showing the blade through
// them, rims glowing where the polyps breathe
vec4 sig_solid(vec4 o, vec2 b, float px){
  if (part(0.0) < 0.5) { return o; }
  float w = dr_w(b.y);
  if (w <= 0.0) return o;
  float cov = fillD(abs(b.x) - w, px)*step(-0.4, b.y);
  if (cov <= 0.0) return o;
  // the openings: rounded pores over the unrolled surface
  float u = b.x/max(w, 0.01);
  vec2 q = vec2(asin(clamp(u, -1.0, 1.0))*1.4, b.y*0.6);
  vec2 cell;
  float e = voro(q, cell);
  float pore = 1.0 - smoothstep(0.18, 0.3, length(fract(q) - 0.5 - (h22(floor(q)) - 0.5)*0.4));
  pore *= step(0.25, h21(floor(q) + 1.7));
  float rim = halo(length(fract(q) - 0.5 - (h22(floor(q)) - 0.5)*0.4) - 0.3, 0.05)*step(0.25, h21(floor(q) + 1.7));
  float breathe = 0.5 + 0.5*sin(uTime*1.4 + h21(floor(q))*12.0);
  // the stone: rough, lit from the upper left, rose with sea-green in its hollows
  float nz = fbm3(vec2(q.x*2.0, q.y*1.3));
  float nx = clamp(u, -1.0, 1.0);
  float dif = clamp(0.6 - 0.45*nx + 0.2*sqrt(1.0 - nx*nx), 0.0, 1.0);
  vec3 stone = mix(DR_SEA*0.7, DR_CORAL*1.2, smoothstep(0.35, 0.65, nz + 0.15*halo(e, 0.1)))*(0.55 + 0.75*dif)*(0.8 + 0.5*nz);
  stone += vec3(1.0, 0.8, 0.75)*rim*(0.8 + 1.2*breathe);
  float a = cov*(1.0 - pore*0.95);
  return vec4(mix(o.rgb, stone, a), max(o.a, a));
}

// a soft sea haze round it
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.15, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.0, 0.5, 4.4, DR_SEA*0.6)*0.8*part(1.0);
  fx += fxAura(ro, rd, y0, y1, 2.2, DR_SEA)*0.35*part(1.0);
  if (cl > 0.01) fx += fxShock(ro, rd, px, L*0.55, 1.0 - cl, 6.0, DR_SEA);
  return fx*uFx;
}
