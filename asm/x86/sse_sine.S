// sse_sine.S — x86_64 SSE sine generator with scalar phase accumulator
// Signature:
//   void sse_sine_f32(float* out, float* phase_ptr, float phase_inc, uint32_t n);
//   rdi = out, rsi = phase_ptr, xmm2 = phase_inc, edx = n

#if defined(__APPLE__)
#define GLBL(sym) _##sym
#else
#define GLBL(sym) sym
#endif

.text
.p2align 4

.intel_syntax noprefix

.globl GLBL(sse_sine_f32)
GLBL(sse_sine_f32):
    test    edx, edx
    jz      L_done

    // load *phase -> xmm3
    movss   xmm3, [rsi]

    // Load constants using RIP-relative addressing (required in x86-64)
    movaps  xmm6, xmmword ptr [rip + XMM_TWO_PI]
    movaps  xmm7, xmmword ptr [rip + XMM_INV_TWO_PI]
    movaps  xmm8, xmmword ptr [rip + XMM_C3]
    movaps  xmm9, xmmword ptr [rip + XMM_C5]
    movaps  xmm10, xmmword ptr [rip + XMM_C7]

L_loop:
    // xr = phase
    movaps  xmm0, xmm3

    // --- range reduce xr to [-pi, pi] ---
    movaps  xmm4, xmm0             // y = xr
    mulss   xmm4, xmm7             // y = xr * inv_two_pi
    roundss xmm4, xmm4, 0          // nearest (SSE4.1)
    movaps  xmm5, xmm4             // k
    mulss   xmm5, xmm6             // k * two_pi
    subss   xmm0, xmm5             // xr = xr - k*two_pi

    // --- polynomial sin(x) ≈ x + c3*x^3 + c5*x^5 + c7*x^7 ---
    movaps  xmm1, xmm0             // x
    mulss   xmm1, xmm1             // x2
    movaps  xmm11, xmm1
    mulss   xmm11, xmm0            // x3

    // t = (c7*x2 + c5)
    movaps  xmm12, xmm10           // c7
    mulss   xmm12, xmm1            // c7*x2
    addss   xmm12, xmm9            // + c5
    // t = t*x2 + c3
    mulss   xmm12, xmm1
    addss   xmm12, xmm8            // + c3
    // y = x + x3 * t
    mulss   xmm12, xmm11           // x3 * t
    addss   xmm12, xmm0            // + x

    // store
    movss   [rdi], xmm12
    add     rdi, 4

    // advance phase
    addss   xmm3, xmm2

    dec     edx
    jnz     L_loop

    // final wrap of *phase into [-pi, pi]
    movaps  xmm0, xmm3
    movaps  xmm4, xmm0
    mulss   xmm4, xmm7
    roundss xmm4, xmm4, 0
    movaps  xmm5, xmm4
    mulss   xmm5, xmm6
    subss   xmm0, xmm5
    movss   [rsi], xmm0

L_done:
    ret

// ---- constants (Mach-O const section) ----
.section __DATA,__const
.p2align 4

// each is a 16-byte block; we just read the low lane (SS), but movaps is happy
XMM_TWO_PI:
    .float 6.2831853071795864769, 6.2831853071795864769, 6.2831853071795864769, 6.2831853071795864769
XMM_INV_TWO_PI:
    .float 0.15915494309189533577, 0.15915494309189533577, 0.15915494309189533577, 0.15915494309189533577
XMM_C3:
    .float -0.16666667163372, -0.16666667163372, -0.16666667163372, -0.16666667163372
XMM_C5:
    .float 0.00833333376795, 0.00833333376795, 0.00833333376795, 0.00833333376795
XMM_C7:
    .float -0.000198412701, -0.000198412701, -0.000198412701, -0.000198412701