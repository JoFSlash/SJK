// Thermal Fuse: a thermal detonator's burning fuse, as a blade. Round 20 (Creyon: a better effect):
// the blade is a braided fuse cord of gold strands; a white-hot spitting point burns slowly down it
// from the tip toward the hilt, spraying sparks all round, the cord behind it glowing ember red,
// smouldering and smoking; when the point reaches the hilt the charge pops in a flash and a ring of
// light, a fresh cord glows back up the blade from the hilt (round 20, Creyon: it started again at
// the tip at once, which looked bad) and the new fuse catches at the tip.
// @part Burning fuse
// @part Fuse sparks
// @room 8
// @palette {"corona.rim_cool":[0.5,0.3,0.05],"corona.rim_hot":[1.0,0.8,0.4],"corona.inner":[1.0,0.95,0.8],"core.white":[1,0.98,0.92],"core.fringe_cool":[1.0,0.75,0.3],"core.fringe_hot":[1.0,0.92,0.7],"light.color":[1.0,0.8,0.4],"trail":[1.0,0.8,0.4]}

const float TF_P = 9.0;
// the cycle: the point burns down over its first 78 %, the charge pops, then the cord renews
float tf_q(){ return fract(uTime/TF_P); }
// the burning point's height: from just below the tip down to near the hilt
float tf_y(){ return uLength*(0.95 - 0.82*smoothstep(0.0, 0.78, tf_q())); }
// how much the point shows: catching at the tip, out once it has popped
float tf_on(){ float q = tf_q(); return smoothstep(0.0, 0.04, q)*(1.0 - smoothstep(0.77, 0.8, q)); }
// the pop when the point reaches the hilt (1 just after, fading)
float tf_pop(){ float q = tf_q() - 0.78; return q < 0.0 ? 0.0 : exp(-q*TF_P*3.0); }
// how burnt the cord is at y: above the point while it burns; after the pop a fresh cord glows back
// up from the hilt to the tip
float tf_burnt(float y){
  float q = tf_q();
  float renew = uLength*1.05*smoothstep(0.82, 0.99, q) - 1.0;
  return smoothstep(tf_y(), tf_y() + 1.2, y)*smoothstep(renew - 1.5, renew, y)*step(0.0, q - 0.02);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float l = lvl(g);
  float yf = tf_y();
  // the braided cord: strands twisting round it
  float braid = 0.5 + 0.5*sin(b.y*2.4 + b.x*9.0);
  float strands = smoothstep(0.2, 0.8, braid);
  vec3 cord = mix(vec3(1.0, 0.62, 0.18), vec3(1.0, 0.86, 0.5), strands);
  // behind the point (above it) the cord has burnt: ember red, smouldering
  float burnt = tf_burnt(b.y)*part(0.0);
  float ember = 0.35 + 0.25*vn(vec2(b.y*1.3 - uTime*0.6, b.x*4.0));
  vec3 c = mix(cord, vec3(1.0, 0.22, 0.05)*ember*1.6, burnt);
  // the white-hot point
  float hot = halo(b.y - yf, 0.7)*tf_on()*part(0.0);
  c = mix(c, vec3(1.0, 0.97, 0.9)*1.8, hot*0.8);
  return l*c*0.8*(1.0 + 0.8*tf_pop());
}
vec3 sig_core(vec3 c, vec2 b, float px){
  // the cord's gold in the core too, dimmed to embers where it has burnt
  float burnt = tf_burnt(b.y)*part(0.0);
  float hot = halo(b.y - tf_y(), 0.7)*tf_on()*part(0.0);
  vec3 tint = mix(mix(vec3(1.0, 0.78, 0.42), vec3(1.0, 0.35, 0.12)*0.5, burnt), vec3(1.0), hot);
  return lvl(c)*tint*0.85;
}
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  vec3 fx = vec3(0.0);
  float yf = tf_y();
  // sparks spitting off the burning point all round, and a few hot glints
  float on = tf_on();
  fx += fxSparks(ro, rd, px, yf - 0.3, yf + 0.3, 20.0 + 10.0*sw, 3.2, 2.4, 1.9, vec3(1.0, 0.7, 0.3))*on*part(1.0);
  fx += fxFlare(ro, rd, vec3(0.0, yf, 0.0), px, 0.25, 1.6, 4.0, t*3.0, vec3(1.0, 0.75, 0.35))*0.6*on*part(0.0);
  // smoke curling off the burnt cord, thinning away as the cord renews
  fx += fxSmoke(ro, rd, yf, L, 1.6, 1.4, 3.3, vec3(0.35, 0.3, 0.28))*0.6*(1.0 - smoothstep(0.8, 0.95, tf_q()))*part(0.0);
  // the pop at the hilt end, then it lights again
  float pop = tf_pop();
  fx += fxShock(ro, rd, px, L*0.12, 1.0 - pop, 4.5, vec3(1.0, 0.75, 0.35))*part(1.0);
  return fx*uFx;
}
