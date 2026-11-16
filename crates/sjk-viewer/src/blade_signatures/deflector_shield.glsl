// Deflector Shield: a starship's deflector field wrapped round the blade, from the hilt to the tip.
// A shell of hexagonal shield cells shimmers round it, brighter at its curved edges, each cell
// slowly flickering; every second or so a hit lands somewhere on it: the cells there flash and a
// ripple races out across the field. A swing brightens it; a clash flares the whole shell.
// @part Hex field
// @part Impact flashes
// @room 8
// @palette {"corona.rim_cool":[0.0,0.25,0.45],"corona.rim_hot":[0.25,0.85,1.0],"corona.inner":[0.75,0.97,1.0],"core.white":[0.92,1.0,1.0],"core.fringe_cool":[0.2,0.8,1.0],"core.fringe_hot":[0.75,0.97,1.0],"light.color":[0.25,0.85,1.0],"trail":[0.25,0.85,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  return g;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the shell's light where the ray meets it at p (normal n): hexagonal cells over the unrolled
// shell, brighter where it is seen edge on, and the hits rippling over it
vec3 ds_at(vec3 p, vec3 n, vec3 rd, float R, float y0, float y1, float yb, float yt){
  float t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 c = vec3(0.4, 0.9, 1.0);
  vec3 fx = vec3(0.0);
  float graze = 1.0 - abs(dot(n, rd));
  float shell = 0.35 + 0.65*pow(graze, 2.0);
  // hexagonal cells over the unrolled shell (round its axis, up it)
  float ang = atan(p.x, p.z);
  vec2 u = vec2(ang*R*0.9, p.y*0.5);
  vec2 sA = vec2(1.0, 1.732);
  vec2 cA = floor(u/sA) + 0.5, cB = floor((u - vec2(0.5, 0.866))/sA) + 0.5;
  vec2 qA = u - cA*sA, qB = u - (cB*sA + vec2(0.5, 0.866));
  vec2 q = dot(qA, qA) < dot(qB, qB) ? qA : qB;
  vec2 id = dot(qA, qA) < dot(qB, qB) ? cA : cB + vec2(0.37, 0.71);
  vec2 aq = abs(q);
  float hx = max(aq.x*0.5 + aq.y*0.866, aq.x);
  float edge = halo(0.5 - hx, 0.035);
  float flick = 0.5 + 0.5*sin(t*1.3 + h21(id)*6.2832);
  float lit = max(cl, 0.35*sw);
  float foot = smoothstep(yb - 0.5, yb + 0.8, p.y);
  fx += c*(edge*(0.3 + 0.35*flick) + 0.08*shell)*foot*(1.0 + 1.6*lit)*part(0.0);
  fx += c*pow(graze, 6.0)*0.5*foot*(1.0 + 1.5*lit)*part(0.0);
  // hits: every second or so a bolt lands, the cells there flash and a ripple races out
  for (int i = 0; i < 2; i++){
    float fi = float(i);
    float cyc = t*0.8 + fi*0.5;
    float k = floor(cyc), qq = fract(cyc);
    vec2 at = vec2((h11(k*3.7 + fi) - 0.5)*R*5.6, mix(y0, y1 - 2.0, h11(k*5.3 + fi*2.0))*0.5);
    vec2 du = u - at;
    du.x = mod(du.x + 3.1416*R*0.9, 6.2832*R*0.9) - 3.1416*R*0.9;
    float dist = length(du);
    float flash = exp(-qq*5.0);
    float ripple = halo(abs(dist - qq*4.0), 0.3)*(1.0 - qq);
    fx += (vec3(1.0)*halo(dist, 0.6)*flash*2.0 + c*(edge*halo(dist, 1.8)*flash*3.0 + ripple*(0.5 + edge)*1.4))*foot*part(1.0);
  }
  return fx;
}
// round 19 (Creyon: seen from above it was a flat picture following the camera): the field is a
// true shell round the blade, a cylinder from the hilt closed by a dome over the tip, drawn where
// the ray enters it and, dimmer, where it leaves
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength;
  float y0 = L*0.2, y1 = L*0.97;
  float R = 2.6, yb = 0.5, yt = L + 1.2, yc = yt - R;
  vec3 fx = vec3(0.0);
  float tin = -1.0, tout = -1.0;
  // the cylinder's wall, from just below the hilt's mouth to where the dome begins
  float a = dot(rd.xz, rd.xz), b = dot(ro.xz, rd.xz), cc = dot(ro.xz, ro.xz) - R*R;
  float h = b*b - a*cc;
  if (a > 1e-6 && h > 0.0){
    h = sqrt(h);
    float t0 = (-b - h)/a, t1 = (-b + h)/a;
    float y = ro.y + rd.y*t0;
    if (t0 > 0.0 && y > yb - 0.5 && y < yc) tin = t0;
    y = ro.y + rd.y*t1;
    if (t1 > 0.0 && y > yb - 0.5 && y < yc) tout = t1;
  }
  // the dome
  vec3 oc = ro - vec3(0.0, yc, 0.0);
  float bs = dot(oc, rd), hs = bs*bs - (dot(oc, oc) - R*R);
  if (hs > 0.0){
    hs = sqrt(hs);
    float t0 = -bs - hs, t1 = -bs + hs;
    if (t0 > 0.0 && ro.y + rd.y*t0 >= yc && tin < 0.0) tin = t0;
    if (t1 > 0.0 && ro.y + rd.y*t1 >= yc && tout < 0.0) tout = t1;
  }
  if (tin > 0.0){
    vec3 p = ro + rd*tin;
    vec3 n = p.y < yc ? normalize(vec3(p.x, 0.0, p.z)) : normalize(p - vec3(0.0, yc, 0.0));
    fx += ds_at(p, n, rd, R, y0, y1, yb, yt);
  }
  if (tout > 0.0){
    vec3 p = ro + rd*tout;
    vec3 n = p.y < yc ? normalize(vec3(p.x, 0.0, p.z)) : normalize(p - vec3(0.0, yc, 0.0));
    fx += ds_at(p, n, rd, R, y0, y1, yb, yt)*0.55;
  }
  return fx*uFx;
}
