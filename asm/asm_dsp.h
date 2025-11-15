#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ARM NEON
void neon_mix_f32(float* dst, const float* src, uint32_t n, float gain);
void neon_sine_f32(float* out, float* phase_ptr, float phase_inc, uint32_t n);

// x86 AVX/SSE
void avx_mix_f32(float* dst, const float* src, uint32_t n, float gain);
void sse_sine_f32(float* out, float* phase_ptr, float phase_inc, uint32_t n);

#ifdef __cplusplus
}
#endif
