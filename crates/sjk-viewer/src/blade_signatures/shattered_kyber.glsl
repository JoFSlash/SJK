// Shattered Kyber: a cracked kyber crystal that will not stay whole. Cracks flicker with light
// through the blade; clear shards of crystal float round it all the time, turning slowly, and
// fly further out on a swing; thin splinters of light snap off the cracks, crack lightning
// jumps between them and kyber dust glitters. A clash shatters it for a moment.
// @part Floating shards
// @part Crack light
// @part Splinters
// @part Kyber dust
// @arcs Crack lightning
// @room 8
// @solid 1
// @palette {"corona.rim_cool":[0.3,0.4,0.8],"corona.rim_hot":[0.75,0.85,1.0],"corona.inner":[0.95,0.98,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.6,0.75,1.0],"core.fringe_hot":[0.9,0.95,1.0],"light.color":[0.7,0.82,1.0],"trail":[0.7,0.82,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec2 cell;
  float e = voro(vec2(b.x*2.5, b.y*0.6), cell);
  float fl = 0.6 + 0.4*sin(uTime*5.0 + h21(cell)*20.0);
  float gap = step(0.82, h21(cell + floor(uTime*0.7)))*0.35*part(0.0);
  return g*(0.85 - gap + 1.1*halo(e, 0.05)*fl*part(1.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.75, 0.88, 1.0);
  float leak = 0.6 + 0.4*sin(t*2.3)*sin(t*1.1);
  fx += fxAura(ro, rd, y0, y1, 2.0, c)*0.5*leak*part(1.0);
  // splinters: short needles of light snapping off the cracks, many more on a swing
  fx += fxSparks(ro, rd, px, y0, y1, 12.0 + 14.0*sw, 2.2, 1.1, 5.1, vec3(0.85, 0.92, 1.0))*1.2*part(2.0);
  fx += fxSparks(ro, rd, px, y0, y1, 6.0 + 6.0*cl, 1.2, 1.7, 2.3, vec3(1.0))*part(2.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 24.0, 0.04, 9.2, c)*part(3.0);
  return fx*uFx;
}
// only what circles the blade is drawn in 3D
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 col = vec4(0.0);
  // shards float round the blade at rest too, further out on a swing or a clash
  float out_ = max(0.4, max(sqrt(clamp(uSwing, 0.0, 1.0)), clamp(uClash, 0.0, 1.0)));
  if (uFx < 0.01 || part(0.0) < 0.5) return col;
  float best = 1e5, th;
  for (int i = 0; i < 20; i++){
    float fi = float(i);
    float y = (0.22 + 0.74*fract(fi*0.381 + 0.07))*uLength + 0.5*sin(uTime*0.6 + fi*1.3);
    vec3 c = around(0.9 + 3.2*out_*(0.4 + 0.6*h11(fi*5.1)), h11(fi*1.7) + uTime*(0.03 + 0.03*h11(fi*2.2)), y);
    float d = rayPt(ro, rd, c, th);
    if (d > 1.2) continue;
    // a clear crystal splinter facing the camera, slowly turning: a long thin diamond
    vec3 q = ro + rd*th - c;
    vec3 u = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
    vec3 v = cross(rd, u);
    vec2 a = rot(uTime*(0.3 + 0.5*h11(fi*4.4)) + fi)*vec2(dot(q, u), dot(q, v));
    float sz = 0.3 + 0.25*h11(fi*6.6);
    float k = abs(a.x)*2.2 + abs(a.y)*0.6 - sz;
    float cv = clamp(0.5 - k/max(px, 1e-3), 0.0, 1.0);
    if (cv > 0.0 && th < best){
      best = th;
      float edge = halo(abs(k), 0.05);
      float glint = pow(0.5 + 0.5*sin(uTime*2.0 + fi*3.1), 8.0);
      col = vec4(vec3(0.75, 0.88, 1.0)*(0.55 + 0.7*edge + 0.8*glint) + vec3(0.2, 0.25, 0.35), cv*0.85);
    }
  }
  return col;
}
