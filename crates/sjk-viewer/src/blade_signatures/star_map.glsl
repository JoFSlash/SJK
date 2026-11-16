// Star Map: an old Jedi star chart as a blade. Seven stars light up beside it one after another and
// lines of light link them into a figure; the chart holds, fades and a new one forms, each time a
// different figure (round 15, Creyon: a hyperspace route, a ring round the blade, a triangle, a
// spiral arm, a diamond, a crown and a lightning route, in a shuffled turn). A glint sits on the tip.
// A clash flares it.
// @part Star chart
// @room 8
// @palette {"corona.rim_cool":[0.05,0.08,0.35],"corona.rim_hot":[0.35,0.45,1.0],"corona.inner":[0.8,0.85,1.0],"core.white":[0.92,0.95,1.0],"core.fringe_cool":[0.3,0.4,1.0],"core.fringe_hot":[0.8,0.86,1.0],"light.color":[0.35,0.45,1.0],"trail":[0.38,0.48,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
// round 15 (Creyon: "let it pick other symbols"): star i (0 to 6) of figure `shape` (0 to 6), x
// across the blade in units, y along it from -1 to 1 of the chart's height; `side` mirrors it
vec2 sm_star(float shape, float fi, float k, float side){
  float a = fi/7.0;
  vec2 p;
  if (shape < 0.5){
    // a hyperspace route zigzagging up beside the blade
    p = vec2(1.0 + 1.8*h11(fi*3.1 + k*7.7), -0.9 + 1.8*(fi + h11(fi*1.3 + k))/7.0);
    p.x *= h11(fi*5.7 + k*2.3) < 0.5 ? -1.0 : 1.0;
    return p;
  }
  if (shape < 1.5) return vec2(2.3*sin(6.2832*a), 0.8*cos(6.2832*a));          // a ring round it
  if (shape < 2.5){                                                            // a triangle
    float e = a*3.0, f = fract(e);
    vec2 v0 = vec2(-2.2, -0.75), v1 = vec2(2.2, -0.75), v2 = vec2(0.0, 0.9);
    vec2 pa = e < 1.0 ? v0 : (e < 2.0 ? v1 : v2), pb = e < 1.0 ? v1 : (e < 2.0 ? v2 : v0);
    return mix(pa, pb, f)*vec2(side, 1.0);
  }
  if (shape < 3.5){                                                            // a spiral arm
    float r = 0.5 + 0.36*fi;
    return vec2(side*r*cos(fi*1.05), 0.36*r*sin(fi*1.05)*1.6);
  }
  if (shape < 4.5){                                                            // a diamond
    float e = a*4.0, f = fract(e);
    vec2 v0 = vec2(0.0, -0.95), v1 = vec2(2.0, 0.0), v2 = vec2(0.0, 0.95), v3 = vec2(-2.0, 0.0);
    vec2 pa = e < 1.0 ? v0 : (e < 2.0 ? v1 : (e < 3.0 ? v2 : v3));
    vec2 pb = e < 1.0 ? v1 : (e < 2.0 ? v2 : (e < 3.0 ? v3 : v0));
    return mix(pa, pb, f);
  }
  if (shape < 5.5){                                                            // a crown
    float up = mod(fi, 2.0) < 0.5 ? -0.2 : 0.55;
    return vec2(-2.4 + 0.8*fi, (fi == 0.0 || fi == 6.0) ? -0.75 : up);
  }
  // a lightning route: hard zigzags climbing the blade's one side
  return vec2(side*(mod(fi, 2.0) < 0.5 ? 0.9 : 2.6), -0.95 + 0.3*fi);
}
// whether a figure closes on its first star (the ring, the triangle and the diamond)
float sm_closed(float shape){ return (shape > 0.5 && shape < 2.5) || (shape > 3.5 && shape < 4.5) ? 1.0 : 0.0; }

vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.7, 0.8, 1.0);
  float ta, s;
  float cyc = t*0.12;
  float k = floor(cyc), q = fract(cyc);
  // a different figure each time, all seven in turn in a shuffled order (0, 3, 6, 2, 5, 1, 4)
  float shape = mod(k*3.0, 7.0);
  float side = h11(k*9.1) < 0.5 ? -1.0 : 1.0;
  float yc = mix(y0, y1, 0.5) + (h11(k*4.3) - 0.5)*(y1 - y0)*0.15, hy = (y1 - y0)*0.4;
  vec3 prev = vec3(0.0), first = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) < 4.5){
    for (int i = 0; i < 7; i++){
      float fi = float(i);
      vec2 f2 = sm_star(shape, fi, k, side);
      // round 19: the figure wrapped round the blade on a cylinder, in three dimensions
      vec3 p = around(3.0, f2.x/(6.2832*3.0) + 0.25 + k*0.37, yc + f2.y*hy);
      float on = smoothstep(fi/9.0, fi/9.0 + 0.05, q)*(1.0 - smoothstep(0.85, 1.0, q));
      float d = rayPt(ro, rd, p, ta);
      float tw = 0.75 + 0.25*sin(t*3.0 + fi*2.0);
      fx += (vec3(1.0)*halo(d, max(0.06, px))*1.6 + c*halo(d, 0.4)*0.4)*on*tw;
      if (i > 0){
        float dl = raySeg(ro, rd, prev, p, s);
        float grow = smoothstep(fi/9.0, fi/9.0 + 0.08, q);
        fx += c*halo(dl, max(0.025, px))*0.6*on*step(s, grow);
      } else {
        first = p;
      }
      prev = p;
    }
    // a closed figure: the last line back to its first star
    float dl = raySeg(ro, rd, prev, first, s);
    float grow = smoothstep(7.0/9.0, 7.0/9.0 + 0.06, q);
    fx += c*halo(dl, max(0.025, px))*0.6*step(s, grow)*(1.0 - smoothstep(0.85, 1.0, q))*sm_closed(shape);
  }
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*part(0.0);
  return vec3(0.0);
}
