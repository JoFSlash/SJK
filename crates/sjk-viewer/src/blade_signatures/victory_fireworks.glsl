// Victory Fireworks: the celebrations across the galaxy the night the Empire fell, as a blade.
// Fireworks burst beside a warm white blade in every colour, their sparks spreading and sinking,
// and gold glitter drifts down round it.
// @part Fireworks
// @part Glitter
// @room 8
// @palette {"corona.rim_cool":[0.55,0.4,0.2],"corona.rim_hot":[1.0,0.88,0.6],"corona.inner":[1.0,0.97,0.88],"core.white":[1,1,0.97],"core.fringe_cool":[1.0,0.85,0.55],"core.fringe_hot":[1.0,0.95,0.82],"light.color":[1.0,0.88,0.6],"trail":[1.0,0.88,0.6]}

vec3 victory_fireworks_glow0(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = victory_fireworks_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  float ta;
  if (fxNear(ro, rd, y0, y1) < 6.0){
    for (int j = 0; j < 3; j++){
      float fj = float(j);
      float cyc = t*0.55 + fj*0.37;
      float k = floor(cyc), q = fract(cyc);
      float side = h11(k*2.1 + fj) < 0.5 ? -1.0 : 1.0;
      // round 19: each burst at its own place round the blade, a sphere of sparks in three dimensions
      vec3 c0 = turnY(vec3(side*(2.0 + 1.5*h11(k*3.7 + fj)), mix(y0 + 4.0, y1 - 3.0, h11(k*5.1 + fj)), 0.0), h11(k*1.3 + fj)*0.5);
      vec3 col = hsv(h11(k*7.3 + fj*1.1), 0.7, 1.0);
      float R = 2.2*(1.0 - pow(1.0 - q, 3.0));
      float life = (1.0 - q)*(1.0 - q);
      for (int i = 0; i < 12; i++){
        float a = 6.2832*(float(i)*0.618 + h11(k + fj));
        float ce = 1.0 - 2.0*(float(i) + 0.5)/12.0, se = sqrt(max(1.0 - ce*ce, 0.0));
        vec3 p = c0 + vec3(cos(a)*se, ce, sin(a)*se)*R - vec3(0.0, 1.2*q*q, 0.0);
        float d = rayPt(ro, rd, p, ta);
        fx += (vec3(1.0)*halo(d, max(0.05, px))*1.2 + col*halo(d, 0.22)*0.8)*life*part(0.0);
      }
    }
  }
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.0, 12.0, -0.03, 2.2, vec3(1.0, 0.85, 0.5))*0.6*part(1.0);
  // round 20 (Creyon): a chroma for the blade only; the fireworks and the glitter keep their own
  // colours
  uKeep += fx*uFx*2.40;
  return vec3(0.0);
}
