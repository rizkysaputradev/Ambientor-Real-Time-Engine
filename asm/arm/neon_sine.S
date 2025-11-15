// neon_sine.S — AArch64 NEON sine generator with phase accumulation
// Signature:
// void neon_sine_f32(float* out, float* phase_ptr, float phase_inc, uint32_t n);

#if defined(__APPLE__)
#define GLBL(sym) _##sym
#else
#define GLBL(sym) sym
#endif

.text
.align  4
.global GLBL(neon_sine_f32)
.p2align 2
GLBL(neon_sine_f32):
    // x0 = out
    // x1 = phase_ptr
    // s2 = phase_inc
    // w3 = n

    // load phase (scalar), broadcast to vector p
    ldr         s3, [x1]                // s3 = *phase
    dup         v16.4s, v3.s[0]         // v16 = [phase, phase, phase, phase]

    // constants: two_pi, inv_two_pi, c3, c5, c7, one, stride [0,1,2,3]
    // We'll use v31..v24 for consts
    adrp        x9, Lconsts@PAGE
    add         x9, x9, Lconsts@PAGEOFF
    ldr         q31, [x9, #0]           // v31 = two_pi
    ldr         q30, [x9, #16]          // v30 = inv_two_pi
    ldr         q29, [x9, #32]          // v29 = c3
    ldr         q28, [x9, #48]          // v28 = c5
    ldr         q27, [x9, #64]          // v27 = c7
    ldr         q26, [x9, #80]          // v26 = one
    ldr         q25, [x9, #96]          // v25 = [0,1,2,3] as float
    ldr         q24, [x9, #112]         // v24 = stride4 = [4,4,4,4]

    // gain: not used; pure sine

    // Broadcast phase_inc to vector inc
    dup         v20.4s, v2.s[0]         // v20 = [inc, inc, inc, inc]

    // Prepare per-iter phase lanes: p + inc * [0,1,2,3]
    fmla        v16.4s, v25.4s, v20.4s  // v16 = base lane phases

    // main loop: process 4 outputs per iter
    cbz         w3, 9f

    // compute block_count = n / 4
    mov         w8, w3
    lsrs        w9, w8, #2
    cbz         w9, 6f

1:  // ---- generate 4 samples into out ----
    // range reduction: y = p * inv_two_pi
    fmul        v21.4s, v16.4s, v30.4s  // y = p * inv2pi
    // k = round(y) -> int
    fcvtas      v22.4s, v21.4s          // nearest int as float (still float)
    // x = p - k * two_pi
    fmls        v16.4s, v22.4s, v31.4s  // v16 = x in [-pi,pi]

    // poly sin(x) = x + c3*x^3 + c5*x^5 + c7*x^7
    fmul        v0.4s,  v16.4s, v16.4s  // x2
    fmul        v1.4s,  v0.4s,  v16.4s  // x3
    // t = c7*x2 + c5
    fmul        v2.4s,  v27.4s, v0.4s   // c7*x2
    fadd        v2.4s,  v2.4s,  v28.4s  // + c5
    // t = t*x2 + c3
    fmul        v2.4s,  v2.4s,  v0.4s
    fadd        v2.4s,  v2.4s,  v29.4s  // + c3
    // y = x + x3 * t
    fmul        v2.4s,  v2.4s,  v1.4s   // x3 * (c7*x2 + c5)*x2 + c3)*x3
    fadd        v2.4s,  v2.4s,  v16.4s  // + x

    // store 4 samples
    st1         {v2.4s}, [x0], #16

    // advance vector phase by inc * 4: p += inc * [4,4,4,4]
    fmla        v16.4s, v24.4s, v20.4s

    // next
    subs        w9, w9, #1
    b.ne        1b

6:  // ---- scalar tail n%4 ----
    ands        w10, w8, #3
    cbz         w10, 8f

7:  // scalar one-by-one
    // range reduction for scalar phase in s4 (move v16 lane0 to s4)
    // Build scalar x = current lane0
    mov         v3.s[0], v16.s[0]       // s3 = p0
    fmul        s4, s3, v30.s[0]        // y = p*inv2pi
    fcvtas      s5, s4                  // k (float)
    fmls        s3, s5, v31.s[0]        // x = p - k*two_pi

    // poly
    fmul        s0, s3, s3              // x2
    fmul        s1, s0, s3              // x3
    fmul        s2, s0, v27.s[0]        // c7*x2
    fadd        s2, s2, v28.s[0]        // + c5
    fmul        s2, s2, s0              // * x2
    fadd        s2, s2, v29.s[0]        // + c3
    fmul        s2, s2, s1              // * x3
    fadd        s2, s2, s3              // + x

    str         s2, [x0], #4

    // advance scalar p0 by inc
    fadd        s6, v16.s[0], v20.s[0]
    mov         v16.s[0], s6

    subs        w10, w10, #1
    b.ne        7b

8:
    // done generating; compute final scalar phase:
    // original base lane0 was (phase + inc*0). We advanced by blocks*4 + tail steps.
    // Compute total increment: inc * n
    scvtf       s7, w3                  // float(n)
    fmul        s7, s7, v20.s[0]        // inc*n
    fadd        s7, s7, v3.s[0]         // phase + inc*n -> s7

    // Wrap to [-pi, pi] for stored phase
    fmul        s4, s7, v30.s[0]        // y = p*inv2pi
    fcvtas      s5, s4
    fmls        s7, s5, v31.s[0]        // s7 = wrapped phase
    str         s7, [x1]                // *phase = s7
    b           9f

9:
    ret

// ---- constants ----
.p2align 4
Lconsts:
    // two_pi
    .quad 0x400921fb54442d18, 0x400921fb54442d18   // 2π as f64 (we'll load as f32? No: use f32)
    // BUT we need f32 vectors; so emit as f32:

    // Re-emit correctly as 4 x f32 per constant block:
    // two_pi (4 x f32)
    .section __DATA,__const
    .p2align 4
Lconsts:
    // v31: two_pi
    .float 6.2831853071795864769, 6.2831853071795864769, 6.2831853071795864769, 6.2831853071795864769
    // v30: inv_two_pi
    .float 0.15915494309189533577, 0.15915494309189533577, 0.15915494309189533577, 0.15915494309189533577
    // v29: c3 = -1/6
    .float -0.16666667163372, -0.16666667163372, -0.16666667163372, -0.16666667163372
    // v28: c5 = 1/120
    .float 0.00833333376795, 0.00833333376795, 0.00833333376795, 0.00833333376795
    // v27: c7 = -1/5040
    .float -0.000198412701, -0.000198412701, -0.000198412701, -0.000198412701
    // v26: one
    .float 1.0, 1.0, 1.0, 1.0
    // v25: [0,1,2,3]
    .float 0.0, 1.0, 2.0, 3.0
    // v24: stride4 [4,4,4,4]
    .float 4.0, 4.0, 4.0, 4.0
