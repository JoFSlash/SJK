// Thousand Cranes: the first Unique blade (Sol, 25/10/2026), a personal thank-you given by hand.
// A thousand paper cranes (senbazuru) are the old Japanese gift of thanks and good wishes. A
// warm silk-white blade with the fine fibres of washi paper in its light, a red thread wound
// round it, and origami cranes in coloured paper circling it as they rise, each wearer's flock
// in its own colours. A golden crane, the thousandth, hovers over the tip. The cranes unfold
// as the blade lights, spread their wings wider on a swing, and a clash startles a ring of
// them out round the blade. A swing releases real paper cranes that climb away from it
// (`@petals`, `saber_petals.rs`).
// @part Red thread
// @part Paper cranes
// @part Golden crane
// @part Washi light
// @part Flying cranes
// @petals {"part":4,"kind":"crane","per_unit":0.035,"life":2.4,"size":2.6,"fall":0.0,"drift":12.0,"rise":42.0}
// @room 10
// @solid 1
// @palette {"corona.rim_cool":[0.78,0.64,0.5],"corona.rim_hot":[1.0,0.92,0.82],"corona.inner":[1.0,0.97,0.92],"core.white":[1.0,0.99,0.96],"core.fringe_cool":[1.0,0.84,0.74],"core.fringe_hot":[1.0,0.94,0.88],"light.color":[1.0,0.9,0.82],"trail":[1.0,0.9,0.84]}

const vec3 TC_SILK = vec3(1.0, 0.95, 0.88);
const vec3 TC_RED = vec3(0.86, 0.12, 0.1);
const vec3 TC_GOLD = vec3(1.0, 0.76, 0.3);

// the paper of a crane: vermilion, indigo, sakura, jade, gold, sky, plum and cream, as a string
// of senbazuru mixes them
vec3 tc_paper(float k){
  float i = floor(mod(k, 8.0));
  if (i < 1.0) return vec3(0.9, 0.24, 0.18);
  if (i < 2.0) return vec3(0.22, 0.3, 0.62);
  if (i < 3.0) return vec3(0.98, 0.66, 0.74);
  if (i < 4.0) return vec3(0.3, 0.66, 0.5);
  if (i < 5.0) return vec3(0.96, 0.76, 0.3);
  if (i < 6.0) return vec3(0.46, 0.68, 0.9);
  if (i < 7.0) return vec3(0.6, 0.32, 0.56);
  return vec3(0.97, 0.93, 0.84);
}

// the signed distance from p to the triangle a, b, c (negative inside)
float tc_tri(vec2 p, vec2 a, vec2 b, vec2 c){
  vec2 e0 = b - a;
  vec2 e1 = c - b;
  vec2 e2 = a - c;
  vec2 v0 = p - a;
  vec2 v1 = p - b;
  vec2 v2 = p - c;
  vec2 q0 = v0 - e0*clamp(dot(v0, e0)/dot(e0, e0), 0.0, 1.0);
  vec2 q1 = v1 - e1*clamp(dot(v1, e1)/dot(e1, e1), 0.0, 1.0);
  vec2 q2 = v2 - e2*clamp(dot(v2, e2)/dot(e2, e2), 0.0, 1.0);
  float s = sign(e0.x*e2.y - e0.y*e2.x);
  vec2 d = min(min(vec2(dot(q0, q0), s*(v0.x*e0.y - v0.y*e0.x)),
                   vec2(dot(q1, q1), s*(v1.x*e1.y - v1.y*e1.x))),
                   vec2(dot(q2, q2), s*(v2.x*e2.y - v2.y*e2.x)));
  return -sqrt(d.x)*sign(d.y);
}

// an origami crane seen from its side in its own plane (x toward its head, y up, about 2 across
// with its wings out), its wings at flap (1 up, -1 down), folded from paper: the far wing in
// shadow, the body with its crease, the neck and tail rising to their points, the head folded
// down, the near wing lit with a fold from its root; rgb and how much it covers
vec4 tc_crane(vec2 q, float flap, vec3 paper, float px){
  if (abs(q.x) > 1.1 || abs(q.y) > 1.1) return vec4(0.0);
  float body = min(tc_tri(q, vec2(-0.46, 0.0), vec2(0.0, 0.13), vec2(0.46, 0.0)),
                   tc_tri(q, vec2(-0.46, 0.0), vec2(0.46, 0.0), vec2(0.0, -0.17)));
  float neck = tc_tri(q, vec2(0.14, 0.04), vec2(0.36, -0.03), vec2(0.7, 0.46));
  float head = tc_tri(q, vec2(0.63, 0.39), vec2(0.72, 0.48), vec2(0.88, 0.35));
  float tail = tc_tri(q, vec2(-0.14, 0.04), vec2(-0.36, -0.03), vec2(-0.76, 0.42));
  float far = tc_tri(q, vec2(-0.22, 0.08), vec2(0.36, 0.08), vec2(0.12, 0.06 + 0.86*flap));
  float wing = tc_tri(q, vec2(-0.36, 0.06), vec2(0.3, 0.06), vec2(-0.02, 0.06 + 0.94*flap));
  float af = fillD(far, px);
  float ab = fillD(min(min(body, neck), min(head, tail)), px);
  float aw = fillD(wing, px);
  vec3 col = paper*0.58;
  // the neck and tail a shade darker than the body, folded under it
  vec3 mid = paper*mix(0.8, 0.94, fillD(body, px));
  // the body's crease, a darker fold along its middle
  mid *= 1.0 - 0.28*(1.0 - smoothstep(0.0, 0.025 + px, abs(q.y - 0.01)))*step(abs(q.x), 0.42);
  col = mix(col, mid, ab);
  // the near wing, lit, with a fold from its root to its tip
  vec3 lit = min(paper*1.12 + 0.07, vec3(1.0));
  float fold = 1.0 - smoothstep(0.0, 0.02 + px, sdSeg(q, vec2(0.0, 0.06), vec2(-0.02, 0.06 + 0.94*flap)));
  col = mix(col, lit*(1.0 - 0.22*fold), aw);
  return vec4(col, max(max(af, ab), aw));
}

// the crane at p, flying along dir (in the blade's frame), size across, wings at flap, drawn
// facing the camera and turned to fly the way it goes across the view; over what res holds
vec4 tc_place(vec4 res, vec3 ro, vec3 rd, float px, vec3 p, vec3 dir, float size, float flap, vec3 paper){
  if (size < 0.05) return res;
  float t;
  float d = rayPt(ro, rd, p, t);
  if (d > size*1.2) return res;
  vec3 fu = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
  vec3 fw = cross(fu, rd);
  vec3 q3 = ro + rd*t - p;
  vec2 q = vec2(dot(q3, fu), dot(q3, fw))/size;
  // flying across the view it shows its side; toward or away from it, it narrows
  float across = dot(dir, fu);
  q.x /= sign(across + 1e-4)*max(abs(across), 0.35);
  // its climb tilts it up
  q = rot(-0.35*dot(dir, fw))*q;
  vec4 c = tc_crane(q, flap, paper, px/size);
  c.rgb = min(c.rgb*(0.9 + 0.45*exp(-length(p.xz)/4.0)) + vec3(0.05, 0.045, 0.04), vec3(1.0));
  return vec4(mix(res.rgb, c.rgb, c.a), max(res.a, c.a));
}

// how far the cranes have unfolded: as the blade lights, they fold into being
float tc_unfold(){ return uAge >= 2000.0 ? 1.0 : smoothstep(0.35, 1.3, uAge); }

// the flock: cranes circling the blade as they rise up it, beating their wings and gliding by
// turns, each wearer's flock in its own order of colours; a swing spreads them wider
vec4 tc_flock(vec3 ro, vec3 rd, float px){
  float L = uLength, sw = clamp(uSwing, 0.0, 1.0);
  float y0 = L*0.12, y1 = L*0.92;
  vec4 res = vec4(0.0);
  if (fxNear(ro, rd, y0, y1) > 9.5) return res;
  float grow = tc_unfold();
  for (int i = 0; i < 8; i++){
    float fi = float(i);
    float rise = fract(uTime*0.022*(0.8 + 0.4*h11(fi*2.3)) + fi/8.0 + 0.05*h11(fi*7.1));
    float a = h11(fi*4.7) + uTime*(0.055 + 0.02*h11(fi*1.9));
    float R = 3.3 + 1.4*h11(fi*3.3) + 1.1*sw + 0.25*sin(uTime*0.6 + fi*1.7);
    float y = mix(y0, y1, rise) + 0.4*sin(uTime*0.9 + fi*2.9);
    vec3 p = around(R, a, y);
    // the way it flies: round the blade and a little up
    float an = 6.2832*a;
    vec3 dir = normalize(vec3(cos(an), 0.18, -sin(an)));
    // beating its wings, then gliding with them out
    float beat = sin(uTime*(5.5 + 2.0*h11(fi*5.9)) + fi*1.3);
    float glide = smoothstep(-0.2, 0.6, sin(uTime*0.45 + fi*2.1));
    float flap = mix(beat, 0.25 + 0.15*beat, glide)*(1.0 + 0.3*sw);
    float life = smoothstep(0.0, 0.07, rise)*(1.0 - smoothstep(0.9, 1.0, rise));
    float size = (2.6 + 0.6*h11(fi*8.3))*grow*life;
    vec3 paper = tc_paper(fi + floor(uSeed*8.0 + 0.5) + 3.0*h11(uSeed*13.0 + fi));
    res = tc_place(res, ro, rd, px, p, dir, size, flap, paper);
  }
  return res;
}

// a clash startles a ring of cranes out round the blade, flying away from it as it rings
vec4 tc_startled(vec3 ro, vec3 rd, float px){
  float cl = clamp(uClash, 0.0, 1.0);
  vec4 res = vec4(0.0);
  if (cl <= 0.02) return res;
  float L = uLength, away = 1.0 - cl;
  for (int i = 0; i < 8; i++){
    float fi = float(i);
    float a = fi/8.0 + 0.06*h11(fi*3.7);
    vec3 p = around(1.2 + away*5.5, a, L*0.55 + away*(1.5 + 2.0*h11(fi*5.1)));
    float an = 6.2832*a;
    vec3 dir = normalize(vec3(sin(an), 0.5, cos(an)));
    float flap = sin(uTime*14.0 + fi*2.0);
    vec3 paper = tc_paper(fi*3.0 + floor(uSeed*8.0 + 0.5));
    res = tc_place(res, ro, rd, px, p, dir, 2.0*smoothstep(0.0, 0.3, cl), flap, paper);
  }
  return res;
}

// the golden crane: the thousandth, hovering over the tip as it circles it slowly, a glint
// running over its gold
vec3 tc_gold_at(){
  float a = uTime*0.07;
  return vec3(0.0, uLength + 2.6 + 0.3*sin(uTime*1.1), 0.0) + around(0.6, a, 0.0);
}
vec4 tc_golden(vec3 ro, vec3 rd, float px){
  vec3 p = tc_gold_at();
  float an = 6.2832*uTime*0.07;
  vec3 dir = normalize(vec3(cos(an), 0.1, -sin(an)));
  float flap = 0.3 + 0.55*sin(uTime*2.2);
  float glint = 0.5 + 0.5*sin(uTime*1.6 + dot(p - ro, vec3(0.4, 0.3, 0.2)));
  vec3 gold = mix(vec3(0.82, 0.56, 0.18), vec3(1.0, 0.9, 0.55), glint);
  return tc_place(vec4(0.0), ro, rd, px, p, dir, 3.2*tc_unfold(), flap, gold);
}

// the blade: Sol's glow from the file in warm silk white, the fine fibres of washi paper drawn
// through its light, and a soft blush of gold breathing up it
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  vec3 col = recol(g, TC_SILK)*1.1;
  float fibre = fbm3(vec2(b.x*5.0/max(r, 0.2), b.y*0.4 - uTime*0.05)) - 0.5;
  float flecks = smoothstep(0.82, 0.95, vn(vec2(b.x*7.0/max(r, 0.2), b.y*1.6)));
  col *= 1.0 + (0.22*fibre + 0.18*flecks)*part(3.0);
  float blush = 0.5 + 0.5*sin(b.y*0.15 - uTime*0.7);
  col += l*TC_GOLD*blush*0.12*part(3.0);
  return col;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*vec3(1.0, 0.97, 0.92); }

// the red thread wound round the blade from the hilt to near the tip: drawn where it passes in
// front, faint where it passes behind the blade's light, turning slowly up it
vec4 sig_solid(vec4 o, vec2 b, float px){
  float L = uLength;
  if (b.y < 0.6 || b.y > L*0.95) return o;
  float A = 0.78, k = 6.2832/9.0;
  float ph = k*b.y - uTime*0.25;
  float x = A*sin(ph);
  float slope = A*k*cos(ph);
  float d = abs(b.x - x)/sqrt(1.0 + slope*slope);
  float front = cos(ph);
  float w = 0.085 + 0.02*front;
  float cov = fillD(d - w, px)*mix(0.3, 1.0, smoothstep(-0.25, 0.25, front));
  cov *= smoothstep(0.6, 1.6, b.y)*(1.0 - smoothstep(L*0.9, L*0.95, b.y))*part(0.0);
  if (cov <= 0.0) return o;
  // a silk thread: a lighter sheen along its middle, darker at its edges
  vec3 rgb = TC_RED*(0.75 + 0.45*(1.0 - smoothstep(0.0, w, d)));
  return vec4(mix(o.rgb, rgb, cov), max(o.a, cov));
}

// the golden crane's warm light round it, from every side (`rayPt` would cut a hole the
// blade's shape in it)
vec3 sig_orbit(vec3 ro, vec3 rd, float px){
  float t;
  float d = rayPtRaw(ro, rd, tc_gold_at(), t);
  return TC_GOLD*halo(d, 2.4)*0.28*tc_unfold()*part(2.0)*uFx;
}

// what circles the blade, in three dimensions: the flock, the startled cranes and the golden
// crane over the tip
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 flock = tc_flock(ro, rd, px)*part(1.0);
  vec4 startled = tc_startled(ro, rd, px)*part(1.0);
  vec4 gold = tc_golden(ro, rd, px)*part(2.0);
  vec4 res = vec4(mix(flock.rgb, startled.rgb, startled.a), max(flock.a, startled.a));
  return vec4(mix(res.rgb, gold.rgb, gold.a), max(res.a, gold.a))*uFx;
}
