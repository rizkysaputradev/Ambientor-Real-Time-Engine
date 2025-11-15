// neon_mix.S — AArch64 NEON mixer
// dst[i] = dst[i] + src[i] * gain, i = 0..n-1
// Signature (AArch64 ABI):
// void neon_mix_f32(float* dst, const float* src, uint32_t n, float gain);
//
// Args:
//   x0 = dst
//   x1 = src
//   w2 = n (count of floats)
//   s3 = gain (float, in v3.s[0])
//
// Works on Apple (Mach-O) and Linux/ELF. We provide symbol macro for both.

#if defined(__APPLE__)
#define GLBL(sym) _##sym
#else
#define GLBL(sym) sym
#endif

.text
.align  4
.global GLBL(neon_mix_f32)
.p2align 2
GLBL(neon_mix_f32):
    // Save gain -> v0.4s (broadcast)
    dup         v0.4s, v3.s[0]          // v0 = [gain gain gain gain]

    // If n < 1, return
    cbz         w2, 1f

    // Process 16 floats per loop (4 * 4-lane vectors)
    // Compute main loop count: m = n / 16
    mov         w8, w2
    lsrs        w9, w8, #4              // w9 = n >> 4 (blocks of 16)
    cbz         w9, 3f                  // if no blocks of 16, go to tail4

2:  // ---- main loop over 16 floats ----
    // load 4x src + 4x dst
    ld1         {v1.4s}, [x1], #16
    ld1         {v2.4s}, [x0]           // dst0
    fmla        v2.4s, v1.4s, v0.4s
    st1         {v2.4s}, [x0], #16

    ld1         {v1.4s}, [x1], #16
    ld1         {v2.4s}, [x0]
    fmla        v2.4s, v1.4s, v0.4s
    st1         {v2.4s}, [x0], #16

    ld1         {v1.4s}, [x1], #16
    ld1         {v2.4s}, [x0]
    fmla        v2.4s, v1.4s, v0.4s
    st1         {v2.4s}, [x0], #16

    ld1         {v1.4s}, [x1], #16
    ld1         {v2.4s}, [x0]
    fmla        v2.4s, v1.4s, v0.4s
    st1         {v2.4s}, [x0], #16

    subs        w9, w9, #1
    b.ne        2b

3:  // ---- tail over groups of 4 floats ----
    ands        w9, w8, #15             // w9 = n % 16
    cbz         w9, 1f
    // t4 = w9 / 4
    lsrs        w10, w9, #2             // blocks of 4
    cbz         w10, 4f

5:  // 4-float chunk
    ld1         {v1.4s}, [x1], #16
    ld1         {v2.4s}, [x0]
    fmla        v2.4s, v1.4s, v0.4s
    st1         {v2.4s}, [x0], #16
    subs        w10, w10, #1
    b.ne        5b

4:  // ---- scalar tail (n % 4) ----
    ands        w11, w9, #3
    cbz         w11, 1f

6:  // scalar: load src, dst; dst += src * gain; store
    ldr         s1, [x1], #4            // src
    ldr         s2, [x0]                // dst
    fmul        s1, s1, s0              // src * gain; s0 == v0.s[0]
    fadd        s2, s2, s1
    str         s2, [x0], #4
    subs        w11, w11, #1
    b.ne        6b

1:
    ret
