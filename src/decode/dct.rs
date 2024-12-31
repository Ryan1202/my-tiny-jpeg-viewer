use std::f32::consts::PI;

#[repr(align(32))]
pub struct DCT {
    pub idct_data: [[f32; 8]; 8],
}

fn cc(x: usize) -> f32 {
    if x == 0 {
        return 1.0 / (2.0 as f32).sqrt();
    } else {
        return 1.0;
    }
}

impl DCT {
    pub fn new() -> DCT {
        let mut output = [[0.0f32; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let cos_term = ((2.0 * i as f32 + 1.0) * j as f32 * PI / 16.0).cos();
                output[i][j] = cc(j) * cos_term / 2.0;
            }
        }
        DCT { idct_data: output }
    }

    pub fn idct(&self, data: [f32; 8]) -> [f32; 8] {
        let mut tmp: [f32; 8] = Default::default();
        for i in 0..8 {
            for j in 0..8 {
                tmp[i] += self.idct_data[i][j] * data[j];
            }
        }
        tmp
    }
    
    #[cfg(target_feature = "avx")]
    pub fn idct2d(&self, data: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
        use std::arch::x86_64::*;
        let mut tmp: [[f32; 8]; 8] = Default::default();
        let mut result: [[f32; 8]; 8] = Default::default();

        for i in 0..8 {
            for j in 0..8 {
                unsafe {
                    // 加载 8 个元素
                    let row = _mm256_loadu_ps(&data[i][0]); // 从 data 的第 i 行加载 8 个元素
                    let coeff = _mm256_loadu_ps(&self.idct_data[j][0]); // 加载系数矩阵
                    let mul = _mm256_mul_ps(row, coeff); // 按元素相乘
                    // 将累加结果水平相加
                    let low = _mm256_extractf128_ps(mul, 0);
                    let high = _mm256_extractf128_ps(mul, 1);
                    let sum_h = _mm_add_ps(low, high);
                    let sum_h = _mm_hadd_ps(sum_h, sum_h);
                    tmp[j][i] = _mm_cvtss_f32(_mm_hadd_ps(sum_h, sum_h));
                }
            }
        }
        
        for i in 0..8 {
            for j in 0..8 {
                unsafe {
                    // 加载 8 个元素
                    let row = _mm256_loadu_ps(&tmp[i][0]); // 从 data 的第 i 行加载 8 个元素
                    let coeff = _mm256_loadu_ps(&self.idct_data[j][0]); // 加载系数矩阵
                    let mul = _mm256_mul_ps(row, coeff); // 按元素相乘
                    // 将累加结果水平相加
                    let low = _mm256_extractf128_ps(mul, 0);
                    let high = _mm256_extractf128_ps(mul, 1);
                    let sum_h = _mm_add_ps(low, high);
                    let sum_h = _mm_hadd_ps(sum_h, sum_h);
                    result[j][i] = _mm_cvtss_f32(_mm_hadd_ps(sum_h, sum_h));
                }
            }
        }

        result
    }
    
    // 自动向量化的代码已经比我写的sse代码好了，就不写sse版本了

    #[cfg(not(target_feature = "avx"))]
    pub fn idct2d(&self, data: [[f32; 8]; 8]) -> [[f32; 8]; 8] {
        let mut tmp: [[f32; 8]; 8] = Default::default();
        let mut result: [[f32; 8]; 8] = Default::default();
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    tmp[j][i] += self.idct_data[j][k] * data[i][k]; // 计算时将data的行列互换
                }
            }
        }        
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    result[j][i] += self.idct_data[j][k] * tmp[i][k]; // 计算时将data的行列互换
                }
            }
        }
        result
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idct() {
        let dct = DCT::new();
        let input = [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0];
        let expected_output = [83.16255985,78.72226374,69.4820188,59.70970069,59.2857482,75.4908429,102.98820747,124.52532417];
        let output = dct.idct(input);
        for i in 0..8 {
            print!("{}, ", output[i]);
            assert!((output[i] - expected_output[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_idct2d() {
        let dct = DCT::new();
        let input = [[231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0],
                     [231.0, -32.0, 48.0, -12.0, 0.0, 0.0, 0.0, 0.0]];
        let output1 = dct.idct2d(input);
        let output2 = dct.idct2d(input);
        for i in 0..8 {
            for j in 0..8 {
                assert!((output1[i][j] - output2[i][j]).abs() < 0.0002);
            }
        }
    }
}