#[cfg(not(target_feature = "sse2"))]
pub fn ycbcr2rgb(
    y: &[f32], y_fact: usize,
    cb: &[f32], cb_fact: usize,
    cr: &[f32], cr_fact: usize,
    buf: &mut [u8]
) {
    let _y = [y[7/y_fact], y[6/y_fact], y[5/y_fact], y[4/y_fact], y[4/y_fact], y[3/y_fact], y[2/y_fact], y[1/y_fact]];
    let _cb = [cb[7/cb_fact], cb[6/cb_fact], cb[5/cb_fact], cb[4/cb_fact], cb[4/cb_fact], cb[3/cb_fact], cb[2/cb_fact], cb[1/cb_fact]];
    let _cr = [cr[7/cr_fact], cr[6/cr_fact], cr[5/cr_fact], cr[4/cr_fact], cr[4/cr_fact], cr[3/cr_fact], cr[2/cr_fact], cr[1/cr_fact]];
    for x in 0..8 {
        buf[x * 4 + 0] = (128.0 + y + 1.402 * cr).round() as u8;
        buf[x * 4 + 1] = (128.0 + y - 0.714 * cr - 0.344 * cb).round() as u8;
        buf[x * 4 + 2] = (128.0 + y + 1.772 * cb).round() as u8;
        buf[x * 4 + 3] = 0xff;
    }
}

#[cfg(target_feature = "sse2")]
pub fn ycbcr2rgb(
    y: &[f32], y_fact: usize,
    cb: &[f32], cb_fact: usize,
    cr: &[f32], cr_fact: usize,
    buf: &mut [u8]
) {
    use std::arch::x86_64::*;
    unsafe {
        let zero = _mm_setzero_ps();
        let u8max = _mm_set1_epi32(255);
        let offset = _mm_set1_ps(128.0);
        let kr1 = _mm_set1_ps(1.402);
        let kg1 = _mm_set1_ps(0.714);
        let kg2 = _mm_set1_ps(0.344);
        let kb1 = _mm_set1_ps(1.772);

        let y0 = _mm_add_ps(_mm_set_ps(y[4/y_fact], y[3/y_fact], y[2/y_fact], y[1/y_fact]), offset);
        let y1 = _mm_add_ps(_mm_set_ps(y[7/y_fact], y[6/y_fact], y[5/y_fact], y[4/y_fact]), offset);
        let cb0 = _mm_set_ps(cb[4/cb_fact], cb[3/cb_fact], cb[2/cb_fact], cb[1/cb_fact]);
        let cb1 = _mm_set_ps(cb[7/cb_fact], cb[6/cb_fact], cb[5/cb_fact], cb[4/cb_fact]);
        let cr0 = _mm_set_ps(cr[4/cr_fact], cr[3/cr_fact], cr[2/cr_fact], cr[1/cr_fact]);
        let cr1 = _mm_set_ps(cr[7/cr_fact], cr[6/cr_fact], cr[5/cr_fact], cr[4/cr_fact]);

        let r0 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_add_ps(y0, _mm_mul_ps(cr0, kr1)), zero
                )
            ),u8max);
        let r1 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_add_ps(y1, _mm_mul_ps(cr1, kr1)), zero
                )
            ),u8max);

        let g0 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_sub_ps(y0,
                        _mm_add_ps(_mm_mul_ps(cr0, kg1), _mm_mul_ps(cb0, kg2))
                    ),
                    zero
                )
            ), u8max);
        let g1 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_sub_ps(y1,
                        _mm_add_ps(_mm_mul_ps(cr1, kg1), _mm_mul_ps(cb1, kg2))
                    ),
                    zero
                )
            ), u8max);

        let b0 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_add_ps(y0, _mm_mul_ps(cb0, kb1)),
                    zero
                )
            ), u8max);
        let b1 = _mm_min_epi32(
            _mm_cvtps_epi32(
                _mm_max_ps(
                    _mm_add_ps(y1, _mm_mul_ps(cb1, kb1)),
                    zero
                )
            ), u8max);
        
        let rgb0 = _mm_or_si128(_mm_or_si128(r0, _mm_slli_epi32(g0, 8)), _mm_slli_epi32(b0, 16));
        let rgb1 = _mm_or_si128(_mm_or_si128(r1, _mm_slli_epi32(g1, 8)), _mm_slli_epi32(b1, 16));

        _mm_storeu_si128(buf[0..15].as_mut_ptr() as *mut __m128i, rgb0);
        buf[3] = 0xff;
        buf[7] = 0xff;
        buf[11] = 0xff;
        buf[15] = 0xff;
        _mm_storeu_si128(buf[16..31].as_mut_ptr() as *mut __m128i, rgb1);
        buf[19] = 0xff;
        buf[23] = 0xff;
        buf[27] = 0xff;
        buf[31] = 0xff;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        let y:[f32;8] = [127.0,127.0,127.0,127.0,127.0,127.0,127.0,127.0];
        let cb:[f32;4] = [-1.0272651, -1.391968, -1.8156782, -1.8800913];
        let cr:[f32;4] = [-2.6093392, -2.444171, -2.1389794, -1.7402275];

        let mut buf:[u8;32] = [0;32];
        super::ycbcr2rgb(&y, 1, &cb, 2, &cr, 2, &mut buf);
        print!("{:?}", buf);
    }
}