use std::f32::consts::PI;

#[repr(align(32))]
pub struct DCT {
    pub idct2d_data: [[[[f32; 8]; 8]; 8]; 8],
}

fn cc(i: usize, j: usize) -> f32 {
    if i == 0 && j == 0 {
        return 1.0 / 2.0;
    } else if i == 0 || j == 0 {
        return 1.0 / (2.0 as f32).sqrt();
    } else {
        return 1.0;
    }
}

impl DCT  {
    pub fn new() -> DCT {
        let mut tmp: [[[[f32; 8]; 8]; 8]; 8] = Default::default();
        for i in 0..8 {
            for j in 0..8 {
                for x in 0..8 {
                    let i_cos = ((2*i+1) as f32 * PI / 16.0 * x as f32).cos();
                    for y in 0..8 {
                        let j_cos =((2*j+1) as f32 * PI / 16.0 * y as f32).cos();
                        tmp[i][j][x][y] = cc(x, y) * i_cos * j_cos / 4.0;
                    }
                }
            }
        }
        DCT { idct2d_data: tmp }
    }

    pub fn idct2d(&self, data: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
        let mut tmp: [[f32; 8]; 8] = Default::default();
        for i in 0..8 {
            for j in 0..8 {
                tmp[i][j] = self.fmd_simd(data, i, j);
            }
        }
        tmp
    }

    #[cfg(target_arch = "x86_64")]
    fn fmd_simd(&self, data: [[f32; 8]; 8], i: usize, j: usize) -> f32 {
        use std::arch::x86_64::*;

        unsafe {
            let mut acc = _mm256_set1_ps(0.0);
            for x in (0..8).step_by(2) {
                let a = _mm256_load_ps(&self.idct2d_data[i][j][x][0]);
                let b = _mm256_loadu_ps(&data[x][0]);
                let c = _mm256_load_ps(&self.idct2d_data[i][j][x+1][0]);
                let d = _mm256_loadu_ps(&data[x+1][0]);
                acc = _mm256_fmadd_ps(a, b, acc);
                acc = _mm256_fmadd_ps(c, d, acc);
            }
            let mut result = [0.0; 8];
            acc = _mm256_hadd_ps(acc, acc);
            acc = _mm256_hadd_ps(acc, acc);
            _mm256_storeu_ps(&mut result[0], acc);
            result[0] + result[4]
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn fmd_simd(&self, data: [[f32; 8]; 8], i: usize, j: usize) -> f32 {
        let mut tmp = 0.0;
        for x in 0..8 {
            for y in 0..8 {
                tmp += self.idct2d_data[i][j][x][y] * data[x][y];
            }
        }
        tmp
    }
}