const PERM: [u8; 512] = [151,160,137,91,90,15,
  131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
  190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
  88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
  77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
  102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
  135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
  5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
  223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
  129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
  251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
  49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
  138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
  151,160,137,91,90,15,
  131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
  190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
  88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
  77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
  102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
  135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
  5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
  223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
  129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
  251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
  49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
  138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180
];

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad3(hash: u32, x:f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h==12 || h==14 { x } else { z };
    (if h&1 != 0 { -u } else  { u }) + (if h&2 != 0 { -v } else { v })
}

fn noise3(x: f32, y: f32, z: f32) -> f32 {
    let ix0 = x.floor() as i32;
    let iy0 = y.floor() as i32;
    let iz0 = z.floor() as i32;
    let fx0 = x - ix0 as f32;
    let fy0 = y - iy0 as f32;
    let fz0 = z - iz0 as f32;
    let fx1 = fx0 - 1.0;
    let fy1 = fy0 - 1.0;
    let fz1 = fz0 - 1.0;
    let ix1 = (ix0 + 1) & 0xff;
    let iy1 = (iy0 + 1) & 0xff;
    let iz1 = (iz0 + 1) & 0xff;
    let ix0 = ix0 & 0xff;
    let iy0 = iy0 & 0xff;
    let iz0 = iz0 & 0xff;

    let r = fade(fz0);
    let t = fade(fy0);
    let s = fade(fx0);

    let nxy0 = grad3(PERM[ix0 as usize + PERM[iy0 as usize + PERM[iz0 as usize] as usize] as usize] as u32, fx0, fy0, fz0);
    let nxy1 = grad3(PERM[ix0 as usize + PERM[iy0 as usize + PERM[iz1 as usize] as usize] as usize] as u32, fx0, fy0, fz1);
    let nx0 = lerp(r, nxy0, nxy1);

    let nxy0 = grad3(PERM[ix0 as usize + PERM[iy1 as usize + PERM[iz0 as usize] as usize] as usize] as u32, fx0, fy1, fz0);
    let nxy1 = grad3(PERM[ix0 as usize + PERM[iy1 as usize + PERM[iz1 as usize] as usize] as usize] as u32, fx0, fy1, fz1);
    let nx1 = lerp(r, nxy0, nxy1);

    let n0 = lerp(t, nx0, nx1);

    let nxy0 = grad3(PERM[ix1 as usize + PERM[iy0 as usize + PERM[iz0 as usize] as usize] as usize] as u32, fx1, fy0, fz0);
    let nxy1 = grad3(PERM[ix1 as usize + PERM[iy0 as usize + PERM[iz1 as usize] as usize] as usize] as u32, fx1, fy0, fz1);
    let nx0 = lerp(r, nxy0, nxy1);

    let nxy0 = grad3(PERM[ix1 as usize + PERM[iy1 as usize + PERM[iz0 as usize] as usize] as usize] as u32, fx1, fy1, fz0);
    let nxy1 = grad3(PERM[ix1 as usize + PERM[iy1 as usize + PERM[iz1 as usize] as usize] as usize] as u32, fx1, fy1, fz1);
    let nx1 = lerp(r, nxy0, nxy1);

    let n1 = lerp(t, nx0, nx1);

    let res = 0.936 * lerp(s, n0, n1);

    return res;
}

pub trait Noise {
    fn compute(&self, x: f32, y: f32, seed: f32) -> f32;
    fn clone_box(&self) -> Box<dyn Noise>;
}

#[derive(Clone)]
struct Octave {
    n: i32,
    offset: i32,
}

impl Noise for Octave {
    fn compute(&self, x: f32, y: f32, seed: f32 ) -> f32 {
        let mut u = 1.0;
        let mut v = 0.0;
        for _i in 0..self.n {
            v += (1.0 / u) * noise3((x / 1.01) * u, (y / 1.01) * u, seed + (self.offset * 32) as f32);
            u *= 2.0;
        }
        v
    }
    fn clone_box(&self) -> Box<dyn Noise> {
        Box::new(self.clone())
    }
}

pub fn octave(n: i32, offset: i32) -> Box<dyn Noise> {
    let octave = Octave { n, offset };
    Box::new(octave)
}

struct Combined {
    n: Box<dyn Noise>,
    m: Box<dyn Noise>,
}

impl Clone for Combined {
    fn clone(&self) -> Self {
        Combined { n: self.n.clone_box(), m: self.m.clone_box() }
    }
}

impl Noise for Combined {
    fn compute(&self, x: f32, y: f32, seed: f32) -> f32 {
        self.n.compute(x + self.m.compute(x, y, seed), y, seed)
    }
    fn clone_box(&self) -> Box<dyn Noise> {
        Box::new(self.clone())
    }
}

pub fn combined(n: Box<dyn Noise>, m: Box<dyn Noise>) -> Box<dyn Noise> {
    let combined = Combined { n, m };
    Box::new(combined)
}

#[derive(Clone)]
struct Basic {
    offset: i32,
}

impl Noise for Basic {
    fn compute(&self, x: f32, y: f32, seed: f32) -> f32 {
        noise3(x, y, seed + (self.offset * 32) as f32)
    }
    fn clone_box(&self) -> Box<dyn Noise> {
        Box::new(self.clone())
    }
}

pub fn basic(offset: i32) -> Box<dyn Noise> {
    let basic = Basic { offset };
    Box::new(basic)
}

struct ExpScale {
    n: Box<dyn Noise>,
    exp: f32,
    scale: f32,
}

impl Clone for ExpScale {
    fn clone(&self) -> Self {
        ExpScale { n: self.n.clone_box(), exp: self.exp, scale: self.scale }
    }
}

impl Noise for ExpScale {
    fn compute(&self, x: f32, y: f32, seed: f32) -> f32 {
        let n = self.n.compute(x * self.scale, y * self.scale, seed);
        n.signum() * n.abs().powf(self.exp)
    }
    fn clone_box(&self) -> Box<dyn Noise> {
        Box::new(self.clone())
    }
}

pub fn exp_scale(n: Box<dyn Noise>, exp: f32, scale: f32) -> Box<dyn Noise> {
    let exp_scale = ExpScale { n, exp, scale };
    Box::new(exp_scale)
}

