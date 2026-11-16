// Scarif Tide: the turquoise seas of Scarif as a blade (round 16 rework, Creyon: it did not make
// sense). The blade is clear tropical water: a turquoise body with sunlit caustics, the bright web
// of light the sun throws through shallow water, rippling over it, and sparkles of sun on its
// surface. Round it, a column of sea water spirals up the blade in 3D like a waterspout, its edge
// catching the light, bubbles rise beside it wobbling as they go, and sea spray glitters off it,
// flung out on a swing. A clash sends a splash out.
// @part Waterspout
// @part Bubbles and spray
// @room 8
// @palette {"corona.rim_cool":[0.0,0.4,0.45],"corona.rim_hot":[0.3,0.95,0.9],"corona.inner":[0.85,1.0,0.98],"core.white":[1,1,1],"core.fringe_cool":[0.2,0.85,0.85],"core.fringe_hot":[0.75,1.0,0.97],"light.color":[0.3,0.9,0.85],"trail":[0.3,0.9,0.85]}

const vec3 SC_SEA = vec3(0.15, 0.85, 0.85);
const vec3 SC_FOAM = vec3(0.85, 1.0, 1.0);

// the blade: shallow water lit by the sun, a moving web of caustics over turquoise and sparkles
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  vec2 q = vec2(b.x*3.0, b.y*0.45);
  vec2 cell;
  float w1 = voro(q + vec2(0.0, -uTime*0.35) + 0.3*vec2(sin(b.y*0.3 + uTime), 0.0), cell);
  float caust = pow(1.0 - smoothstep(0.0, 0.18, w1), 2.0)*0.8 + (1.0 - smoothstep(0.0, 0.4, w1))*0.15;
  // round 17 (Creyon: it looked laggy): the sun's sparkles twinkle smoothly (they jumped eight
  // times a second), and one web of caustics, softly doubled, instead of two
  vec2 sc = floor(vec2(b.x*16.0, b.y*3.0));
  float sparkle = pow(max(sin(uTime*2.5 + h21(sc)*40.0), 0.0), 24.0)*step(0.8, h21(sc + 3.7))*smoothstep(0.1, 0.4, abs(b.x));
  vec3 c = SC_SEA*(0.75 + 0.2*sin(b.y*0.2 - uTime*0.8)) + SC_FOAM*caust*0.9 + vec3(1.0)*sparkle*1.5;
  return l*c*(1.0 + 0.4*clamp(uClash, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }

// bubbles rising beside it, wobbling, each a thin bright ring with a glint, and sea spray (flat,
// facing the camera, as Sol's are)
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.15, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  float ta;
  if (fxNear(ro, rd, y0, y1) < 5.0){
    for (int i = 0; i < 12; i++){
      float fi = float(i);
      float q = fract(t*(0.12 + 0.08*h11(fi*2.3)) + h11(fi*5.9));
      float side = h11(fi*3.7) < 0.5 ? -1.0 : 1.0;
      vec3 p = vec3(side*(0.9 + 1.6*h11(fi*1.3)) + 0.25*sin(t*3.0 + fi*2.0), mix(y0, y1, q), 0.0);
      float R = 0.12 + 0.22*h11(fi*7.1);
      float d = rayPt(ro, rd, p, ta);
      float ring = halo(abs(d - R), max(0.03, px));
      float glint = halo(length((ro + rd*ta - p).xy - vec2(-0.4, 0.4)*R), R*0.3);
      fx += (SC_FOAM*ring*0.9 + vec3(1.0)*glint*1.2 + SC_SEA*halo(d, R*1.5)*0.15)*sin(3.1416*q)*part(1.0);
    }
  }
  fx += fxSparks(ro, rd, px, y0, y1, 6.0 + 18.0*sw, 3.0, 1.0, 6.1, SC_FOAM)*0.8*part(1.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.0, 16.0, 0.05, 2.2, SC_FOAM)*0.6*part(1.0);
  if (cl > 0.01) fx += fxShock(ro, rd, px, L*0.55, 1.0 - cl, 6.0, SC_SEA)*1.2;
  return fx*uFx;
}

// the waterspout: a sheet of sea water spiralling up round the blade, their leading edges
// catching the sun; they circle the blade, so they are drawn in true 3D (Sol's way)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  if (part(0.0) < 0.5) { return vec3(0.0); }
  float L = uLength, t = uTime, s;
  float y0 = L*0.1, y1 = L*0.95;
  vec3 acc = vec3(0.0);
  if (fxNear(ro, rd, y0, y1) > 4.0) return acc;
  for (int j = 0; j < 1; j++){
    float fj = float(j);
    float best = 1e3, bf = 0.0;
    vec3 prev = vec3(0.0);
    for (int i = 0; i <= 26; i++){
      float f = float(i)/26.0;
      float y = mix(y0, y1, f);
      float R = 1.3 + 0.9*f + 0.15*sin(f*14.0 - t*3.0 + fj*3.0);
      vec3 p = around(R, f*1.6 - t*0.3 + fj*0.5, y);
      if (i > 0){ float d = raySeg(ro, rd, prev, p, s); if (d < best){ best = d; bf = (float(i) - 1.0 + s)/26.0; } }
      prev = p;
    }
    float ends = smoothstep(0.0, 0.08, bf)*(1.0 - smoothstep(0.85, 1.0, bf));
    float crest = pow(0.5 + 0.5*sin(bf*30.0 - t*4.0 + fj*2.0), 4.0);
    acc += (SC_FOAM*halo(best, max(0.04, px))*(0.3 + 1.0*crest) + SC_SEA*halo(best, 0.5)*0.55 + SC_SEA*halo(best, 1.4)*0.2)*ends;
  }
  return acc*uFx;
}
