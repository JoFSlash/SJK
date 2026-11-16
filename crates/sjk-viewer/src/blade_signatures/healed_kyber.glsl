// Healed Kyber: a cracked kyber crystal mended with gold, as a Jedi heals what was broken. A pale
// blade crossed by glowing gold seams, light running slowly along them; a soft healing light
// breathes round it and rings of gold light rise up it slowly; gold dust drifts off it. Every
// clash makes the seams flare.
// @part Gold seams
// @part Healing light
// @part Gold dust
// @room 8
// @palette {"corona.rim_cool":[0.55,0.38,0.1],"corona.rim_hot":[1.0,0.8,0.35],"corona.inner":[1.0,0.93,0.75],"core.white":[1.0,0.98,0.92],"core.fringe_cool":[1.0,0.78,0.35],"core.fringe_hot":[1.0,0.92,0.7],"light.color":[1.0,0.8,0.4],"trail":[1.0,0.78,0.38]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec2 cell;
  float e = voro(vec2(b.x*2.2, b.y*0.45), cell);
  float wave = 0.6 + 0.4*sin(b.y*0.3 - uTime*1.2 + h21(cell)*3.0);
  float seam = halo(e, 0.05)*(wave + 1.5*clamp(uClash, 0.0, 1.0))*part(0.0);
  return mix(recol(g, vec3(0.92, 0.95, 1.0)), lvl(g)*vec3(1.0, 0.75, 0.3)*1.6, clamp(seam, 0.0, 1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 gold = vec3(1.0, 0.78, 0.35);
  float breathe = 0.6 + 0.4*sin(t*0.9);
  fx += fxAura(ro, rd, y0, y1, 2.2, vec3(1.0, 0.95, 0.85))*0.45*breathe*part(1.0);
  fx += fxRingWaves(ro, rd, px, y0, y1, 2.0, 1.8, 2.5, 0.3, gold)*1.3*part(1.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.6, 2.8, 18.0, 0.025, 6.3, gold)*0.9*part(2.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx;
  return vec3(0.0);
}
