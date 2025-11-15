// avx_mix.S — x86_64 AVX mixer
// Signature (SysV AMD64 / macOS):
//   void avx_mix_f32(float* dst, const float* src, uint32_t n, float gain);
//   rdi = dst, rsi = src, edx = n, xmm3 = gain

#if defined(__APPLE__)
#define GLBL(sym) _##sym
#else
#define GLBL(sym) sym
#endif

.text
.p2align 4

// Switch clang's assembler to Intel syntax (so [rsi], dst-first, no size suffixes)
.intel_syntax noprefix

.globl GLBL(avx_mix_f32)
GLBL(avx_mix_f32):
    test    edx, edx
    jz      L_done

    // broadcast gain (xmm3) to ymm0
    vbroadcastss    ymm0, xmm3

    // ecx = n / 8 (process 8 floats per iter)
    mov     ecx, edx
    shr     ecx, 3
    jz      L_tail4

L_loop8:
    vmovups ymm1, [rsi]           // src[0..7]
    vmovups ymm2, [rdi]           // dst[0..7]
    vmulps  ymm1, ymm1, ymm0      // src * gain
    vaddps  ymm2, ymm2, ymm1      // dst += src * gain
    vmovups [rdi], ymm2

    add     rsi, 32               // 8 * 4 bytes
    add     rdi, 32
    dec     ecx
    jnz     L_loop8

L_tail4:
    // remainder = n % 8
    mov     ecx, edx
    and     ecx, 7
    cmp     ecx, 4
    jb      L_tail_scalar

    // handle 4 floats with xmm
    vmovups xmm1, [rsi]
    vmovups xmm2, [rdi]
    vmulps  xmm1, xmm1, xmm0
    vaddps  xmm2, xmm2, xmm1
    vmovups [rdi], xmm2
    add     rsi, 16
    add     rdi, 16
    sub     ecx, 4

L_tail_scalar:
    test    ecx, ecx
    jz      L_done

L_scalar_loop:
    movss   xmm1, [rsi]
    movss   xmm2, [rdi]
    mulss   xmm1, xmm0            // gain in xmm0.s0
    addss   xmm2, xmm1
    movss   [rdi], xmm2
    add     rsi, 4
    add     rdi, 4
    dec     ecx
    jnz     L_scalar_loop

L_done:
    vzeroupper
    ret
