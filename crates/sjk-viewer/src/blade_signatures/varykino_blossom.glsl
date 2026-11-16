// Varykino Blossom: the spring gardens of Varykino, Naboo's lake retreat, as a blade (Kushina's
// idea, 15/10/2026; the Mythical sibling of the Naboo Blossom). A soft blossom-pink blade, its
// palette locked, with pale petals of light opening and fading along it; lavender and white mist
// curls off it, blossoms drift down round it, and a branch of cherry wood with one flower grows
// from where the blade leaves the hilt. Swinging it sheds real blossoms that flutter to the
// ground behind it (`@petals`, `saber_petals.rs`); a clash bursts a ring of petals.
// Sol's addition (21/10/2026), after the Sakura Droplet of Sekiro: a pearl of nacre hangs at
// its tip, rose and lilac with a rainbow sheen and a white heart (a clash bursts it into the
// petals and it grows back), and the blade itself has the same pearly sheen.
// @part Blossom branch
// @part Lavender mist
// @part Drifting petals
// @part Falling blossoms
// @part Sakura droplet
// @petals {"part":3,"per_unit":0.07,"life":2.6,"size":1.6,"fall":38.0,"drift":14.0}
// @room 9
// @solid 1
// @palette {"corona.rim_cool":[0.72,0.18,0.42],"corona.rim_hot":[1.0,0.55,0.75],"corona.inner":[1.0,0.68,0.84],"core.white":[1.0,0.86,0.93],"core.fringe_cool":[1.0,0.5,0.72],"core.fringe_hot":[1.0,0.7,0.84],"light.color":[1.0,0.55,0.75],"trail":[1.0,0.6,0.8]}

const vec3 VB_PINK = vec3(1.0, 0.52, 0.74);
const vec3 VB_PALE = vec3(1.0, 0.86, 0.92);
const vec3 VB_LILAC = vec3(0.76, 0.64, 0.96);
// the droplet's nacre: dusty rose, mauve and cream
const vec3 VB_ROSE = vec3(0.96, 0.6, 0.78);
const vec3 VB_MAUVE = vec3(0.74, 0.48, 0.7);
const vec3 VB_CREAM = vec3(0.96, 0.88, 0.88);
const vec3 VB_WOOD = vec3(0.32, 0.19, 0.12);
const vec3 VB_BARK = vec3(0.58, 0.38, 0.26);

// a five-petal blossom's shape at q (its own plane, radius 1): 0 outside, 1 inside a petal
float vb_flower(vec2 q, float px){
  float a = atan(q.y, q.x);
  float r = length(q);
  // five rounded petals, each with the little notch at its end a cherry blossom has
  float petal = 0.62 + 0.38*abs(cos(2.5*a));
  float notch = 0.12*pow(abs(cos(2.5*a)), 24.0);
  return fillD(r - (petal - notch), px);
}

// the blade: Sol's glow from the file in blossom pink, pale petals of light opening and fading
// along it, a slow breath of white blushing up from the hilt
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  vec3 col = recol(g, VB_PINK)*1.15;
  float blush = 0.5 + 0.5*sin(b.y*0.18 - uTime*0.9);
  col += l*VB_PALE*blush*0.35;
  // petals of light in the glow: cells along the blade, each a soft oval that opens and fades
  float cell = floor(b.y*0.45);
  float f = fract(uTime*0.25 + h11(cell*7.1));
  vec2 c = vec2((h11(cell*3.3) - 0.5)*0.9*r, (cell + 0.5)/0.45);
  vec2 q = rot(h11(cell*5.7)*6.2832 + uTime*0.3)*(b - c);
  float pd = length(q/vec2(0.55, 0.3));
  col += l*VB_PALE*halo(pd - 0.6, 0.25)*sin(3.1416*f)*0.6;
  // nacre: faint growth lines across the blade, and a soft rainbow sheen at its edges that
  // drifts as a pearl's does when it turns
  float rim = smoothstep(0.25, 0.95, abs(b.x)/max(r, 0.001));
  float lines = 0.5 + 0.5*sin(b.y*1.3 + 4.0*b.x*b.x/max(r*r, 0.001) - uTime*0.4);
  vec3 sheen = hsv(fract(b.y*0.04 + b.x*0.25 + uTime*0.05), 0.45, 1.0);
  col += l*(sheen*rim*0.3 + VB_CREAM*lines*0.08);
  return col;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c*vec3(1.0, 0.72, 0.86); }

// blossoms drifting down round the blade: each a five-petal flower facing the camera, turning
// and swaying as it falls, pink or white
vec3 vb_petals(vec3 ro, vec3 rd, float px, float y0, float y1){
  vec3 acc = vec3(0.0);
  if (fxNear(ro, rd, y0 - 4.0, y1) > 4.0) return acc;
  float ta;
  vec3 fu = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
  vec3 fv = cross(rd, fu);
  for (int i = 0; i < 14; i++){
    float fi = float(i);
    float f = fract(uTime*0.07*(0.7 + 0.6*h11(fi*2.7)) + h11(fi*6.1));
    vec3 p = around(0.8 + 2.0*h11(fi*3.9) + 0.5*sin(uTime*0.8 + fi*2.0), h11(fi*5.3) + 0.04*uTime, mix(y1, y0 - 4.0, f));
    float d = rayPt(ro, rd, p, ta);
    if (d > 1.6 + px) continue;
    vec3 q = ro + rd*ta - p;
    // tumbling: the flower's width shrinks and grows as it turns edge on and back
    float tumble = 0.45 + 0.55*abs(sin(uTime*(0.8 + 0.6*h11(fi*1.9)) + fi));
    vec2 e = rot(uTime*0.9 + fi*1.7)*vec2(dot(q, fu), dot(q, fv));
    float size = 0.55 + 0.2*h11(fi*8.3);
    float shape = vb_flower(e/vec2(size, size*tumble), px/size);
    vec3 tint = mix(VB_PINK, VB_PALE, step(0.55, h11(fi*4.4)));
    float life = smoothstep(0.0, 0.08, f)*(1.0 - smoothstep(0.85, 1.0, f));
    acc += (tint*shape*0.9 + tint*halo(d, 0.7)*0.06)*life;
  }
  return acc;
}

// lavender and white mist curling up off the blade, two soft clouds woven together
vec3 vb_mist(vec3 ro, vec3 rd, float y0, float y1){
  return fxSmoke(ro, rd, y0, y1, 2.8, 1.6, 3.0, VB_LILAC)*1.1
    + fxSmoke(ro, rd, y0 + 2.0, y1, 2.0, 2.2, 11.0, vec3(1.0, 0.94, 0.98))*0.6;
}

// the effects round the blade
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.15, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += vb_mist(ro, rd, y0, y1)*part(1.0);
  fx += vb_petals(ro, rd, px, y0, y1)*part(2.0);
  // the droplet's soft rose light, round it from every side (`rayPt` would hide what is behind
  // the blade's body and cut a hole the blade's shape in it)
  float tp;
  float dp = rayPtRaw(ro, rd, vb_pearl_at(), tp);
  fx += VB_PINK*halo(dp, 1.8*max(vb_pearl_size(), 0.2))*0.35*part(4.0);
  // a clash: a ring of petals bursting out across the blade
  if (cl > 0.0){
    float y = L*0.6;
    vec3 ring = vec3(0.0);
    float ta;
    for (int i = 0; i < 10; i++){
      float fi = float(i);
      float R = 0.8 + (1.0 - cl)*4.0;
      vec3 p = around(R, fi/10.0 + 0.05*h11(fi), y + (1.0 - cl)*(h11(fi*2.3) - 0.5)*2.0);
      float d = rayPt(ro, rd, p, ta);
      ring += mix(VB_PINK, VB_PALE, h11(fi*3.1))*(halo(d, 0.28)*0.9 + halo(d, 0.9)*0.15);
    }
    fx += ring*cl*1.3;
  }
  return fx*uFx;
}

// an open blossom at fp, size across, facing the camera, turned by turn: pale at its heart,
// pink at its petals' ends, a gold heart of stamens, over what res holds
vec4 vb_bloom(vec4 res, vec3 ro, vec3 rd, float px, vec3 fp, float size0, float turn){
  float tf;
  float df = rayPt(ro, rd, fp, tf);
  if (df < size0*1.3){
    vec3 q = ro + rd*tf - fp;
    vec3 fu = normalize(cross(rd, abs(rd.y) < 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0)));
    vec2 p = rot(turn + 0.15*sin(uTime*0.7))*vec2(dot(q, fu), dot(q, cross(rd, fu)));
    float size = size0*(1.0 + 0.04*sin(uTime*1.3 + turn));
    float shape = vb_flower(p/size, px/size);
    float r = length(p)/size;
    // pale at its heart, pink at its petals' ends, a gold heart of stamens
    vec3 petal = mix(VB_PALE, VB_PINK, smoothstep(0.15, 0.9, r));
    float heart = fillD(r - 0.18, px/size);
    vec3 col = mix(petal, vec3(1.0, 0.8, 0.35), heart);
    res = vec4(mix(res.rgb, col, shape), max(res.a, shape));
  }
  return res;
}

// the droplet's place: a hanging pearl just past the tip; its size, grown back after a clash
// bursts it
vec3 vb_pearl_at(){ return vec3(0.0, uLength + 0.8, 0.0); }
float vb_pearl_size(){ return 0.95*(1.0 - clamp(uClash, 0.0, 1.0))*(1.0 + 0.04*sin(uTime*1.1)); }

// the Sakura Droplet: a teardrop of nacre facing the camera, its point on the blade's tip, rose
// and lilac with growth rings, a rainbow sheen at its rim and a white glow at its heart
vec4 vb_pearl(vec3 ro, vec3 rd, float px){
  float R = vb_pearl_size();
  if (R < 0.05) return vec4(0.0);
  vec3 c = vb_pearl_at();
  float t;
  float d = rayPt(ro, rd, c, t);
  if (d > R*1.8) return vec4(0.0);
  vec3 q = ro + rd*t - c;
  // the blade's direction as seen, so the point stays toward it from every side
  vec3 up = vec3(0.0, 1.0, 0.0) - rd*rd.y;
  up = length(up) > 1e-3 ? normalize(up) : normalize(cross(rd, vec3(1.0, 0.0, 0.0)));
  vec3 side = cross(rd, up);
  // y toward the blade, so its point touches the tip and its round end hangs off it
  vec2 p = vec2(dot(q, side), -dot(q, up))/R;
  // a teardrop: a round base (radius 1) drawn to a soft point 1.2 above its middle, toward the
  // blade, full in the body as the Sakura Droplet is
  float h = clamp(p.y/1.2, 0.0, 1.0);
  float wide = sqrt(max(1.0 - h*h, 0.0))*(1.0 - 0.3*h*h);
  float dist = p.y < 0.0 ? length(p) - 1.0 : (p.y > 1.2 ? 1.0 : abs(p.x) - wide);
  float shape = fillD(dist, px/R);
  if (shape <= 0.0) return vec4(0.0);
  // how near its edge, as a pearl's surface turns away
  float n = clamp(p.y < 0.0 ? length(p) : abs(p.x)/max(wide, 0.05), 0.0, 1.0);
  float edge = pow(n, 3.0);
  // growth rings gathering toward the base, as the droplet's
  float rings = 0.5 + 0.5*sin(length(p - vec2(0.0, -0.9))*14.0 - uTime*0.3);
  rings *= 1.0 - edge;
  vec3 col = mix(VB_ROSE, VB_MAUVE, edge*0.6) + VB_CREAM*rings*0.12;
  col = mix(col, VB_LILAC, 0.25*smoothstep(0.2, 0.8, p.x + 0.5));
  // the rainbow sheen at its rim, drifting as it turns
  col += hsv(fract(p.x*0.4 + p.y*0.3 + uTime*0.07), 0.7, 1.0)*(edge*0.6 + 0.08);
  // its white heart and a highlight
  col += VB_CREAM*halo(length(p - vec2(0.0, -0.1)), 0.25)*0.45;
  col += vec3(1.0)*halo(length(p - vec2(-0.35, -0.3)), 0.1)*0.7;
  return vec4(min(col, vec3(1.0)), shape*0.92);
}

// the branch: a twig of dark cherry wood growing from where the blade leaves the hilt, curving
// out and up, a smaller twig off it with a bud, and two open blossoms; it circles
// nothing, but stands in true 3D beside the blade
// the branch, its bud and its blossoms
vec4 vb_branch(vec3 ro, vec3 rd, float px){
  if (fxNear(ro, rd, -0.5, 9.0) > 6.5) return vec4(0.0);
  vec3 a = vec3(0.45, 0.3, 0.0);
  vec3 m = vec3(2.1, 3.2, 0.4);
  vec3 e = vec3(3.6, 6.0, 0.25);
  vec3 side = vec3(2.7, 4.3, -1.0);
  float s1, s2, s3;
  float d1 = raySeg(ro, rd, a, m, s1);
  float d2 = raySeg(ro, rd, m, e, s2);
  float d3 = raySeg(ro, rd, m + (e - m)*0.3, side, s3);
  // tapering: thickest at the hilt, thin at the twigs' ends
  float w1 = mix(0.34, 0.24, s1), w2 = mix(0.24, 0.13, s2), w3 = mix(0.15, 0.08, s3);
  float wood = max(max(fillD(d1 - w1, px), fillD(d2 - w2, px)), fillD(d3 - w3, px));
  // bark: lighter along the wood's lit edge, darker rings of grain
  float edge = max(max(d1/w1, d2/w2), 0.0);
  vec3 bark = mix(VB_BARK, VB_WOOD, clamp(edge, 0.0, 1.0))*(0.85 + 0.15*sin((s1 + s2)*14.0));
  vec4 res = vec4(bark, wood);
  // the bud at the side twig's end
  float tb;
  float db = rayPt(ro, rd, side, tb);
  float bud = fillD(db - 0.32, px);
  res = vec4(mix(res.rgb, mix(VB_PINK, VB_PALE, 0.3), bud), max(res.a, bud));
  // the open blossoms: one at the branch's end, a smaller one by its fork
  res = vb_bloom(res, ro, rd, px, e + vec3(0.25, 0.4, 0.0), 1.45, 0.3);
  res = vb_bloom(res, ro, rd, px, m + vec3(-0.2, 0.55, 0.5), 0.95, 2.1);
  return res;
}

// what stands round the blade: the branch at the hilt and the droplet at the tip
vec4 sig_orbit_solid(vec3 ro, vec3 rd, float px){
  vec4 drop = vb_pearl(ro, rd, px)*part(4.0);
  vec4 branch = vb_branch(ro, rd, px)*part(0.0);
  return vec4(mix(branch.rgb, drop.rgb, drop.a), max(branch.a, drop.a));
}
