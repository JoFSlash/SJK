// Holo-Recording: a worn old hologram recording as a blade. It shows in fine scan lines with its
// colours split pink and blue across it, a bright tracking band rolls slowly up it and short
// dashes of static crackle beside it. A clash flares it.
// @part Tracking and static
// @room 8
// @palette {"corona.rim_cool":[0.55,0.14,0.4],"corona.rim_hot":[1.0,0.35,0.7],"corona.inner":[0.6,0.95,1.0],"core.white":[1.0,0.97,0.94],"core.fringe_cool":[0.6,0.15,0.42],"core.fringe_hot":[1.0,0.35,0.7],"light.color":[1.0,0.4,0.72],"trail":[1.0,0.4,0.72]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float yb = mod(uTime*4.0, uLength + 10.0) - 5.0;
  float band = halo(abs(b.y - yb), 1.2)*part(0.0);
  vec3 c = mix(vec3(1.0, 0.45, 0.6), vec3(0.45, 0.8, 1.0), smoothstep(-0.3, 0.3, b.x + 0.2*band));
  return recol(g, c)*(0.75 + 0.25*sin(b.y*8.0))*(1.0 + 0.6*band);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.7, 0.9, 1.0);
  float ta;
  // round 23: out to where the dashes' glow has faded (cut at 3 units, they ended at a hard edge)
  if (fxNear(ro, rd, y0, y1) < 4.0){
    for (int i = 0; i < 14; i++){
      float fi = float(i);
      float k = floor(t*12.0 + fi*0.37);
      // round 20: all round the blade in 3D, each dash level and facing the camera
      vec3 p = around(0.5 + 1.3*h11(k*1.7 + fi), h11(k*4.1 + fi), mix(y0, y1, h11(k*2.9 + fi*3.1)));
      float d = rayPt(ro, rd, p, ta);
      vec3 dq = ro + rd*ta - p;
      vec3 fu = normalize(cross(rd, vec3(0.0, 1.0, 0.0)) + vec3(1e-4, 0.0, 0.0));
      float dash = halo(abs(dq.y), max(0.03, px))*halo(abs(dot(dq, fu)), 0.35);
      fx += c*dash*0.6*step(0.55, h11(k*5.3 + fi));
    }
  }
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*part(0.0);
  return vec3(0.0);
}
