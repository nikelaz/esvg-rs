#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- Plugin flag bits ---- */
#define ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS    (UINT64_C(1) << 0)
#define ESVG_PLUGIN_SHAPE_TO_PATH               (UINT64_C(1) << 1)
#define ESVG_PLUGIN_OPTIMIZE_COLORS             (UINT64_C(1) << 2)
#define ESVG_PLUGIN_COLLAPSE_GROUPS             (UINT64_C(1) << 3)
#define ESVG_PLUGIN_NUMBER_PRECISION            (UINT64_C(1) << 4)
#define ESVG_PLUGIN_REMOVE_EMPTY_TEXT           (UINT64_C(1) << 5)
#define ESVG_PLUGIN_REMOVE_UNNECESSARY_CLIPPATH (UINT64_C(1) << 6)
#define ESVG_PLUGIN_SORT_ATTRS                  (UINT64_C(1) << 7)
#define ESVG_PLUGIN_APPLY_TRANSFORMS            (UINT64_C(1) << 8)
#define ESVG_PLUGIN_CSS_TO_ATTRIBUTES           (UINT64_C(1) << 9)
#define ESVG_PLUGIN_COMBINE_PATHS               (UINT64_C(1) << 10)
#define ESVG_PLUGIN_MANGLE_IDS                  (UINT64_C(1) << 11)
#define ESVG_PLUGIN_SIMPLIFY_PATHS              (UINT64_C(1) << 12)
#define ESVG_PLUGIN_COUNT                       13

/** Default plugin set — matches the original hardcoded behaviour. */
#define ESVG_DEFAULT_FLAGS ( \
    ESVG_PLUGIN_REMOVE_UNNECESSARY_ATTRS | \
    ESVG_PLUGIN_SHAPE_TO_PATH            | \
    ESVG_PLUGIN_OPTIMIZE_COLORS          | \
    ESVG_PLUGIN_APPLY_TRANSFORMS         | \
    ESVG_PLUGIN_CSS_TO_ATTRIBUTES        | \
    ESVG_PLUGIN_COMBINE_PATHS)

/* ---- API ---- */

/**
 * Optimize an SVG string using the default plugin set.
 * @param input  Pointer to UTF-8 SVG bytes (not required to be null-terminated).
 * @param len    Byte count.
 * @return       Heap-allocated null-terminated optimized SVG, or NULL on error.
 *               Must be freed with esvg_free().
 */
char* esvg_optimize(const char* input, size_t len);

/**
 * Optimize an SVG string with an explicit plugin selection bitmask.
 * Use ESVG_PLUGIN_* constants to compose the flags argument.
 */
char* esvg_optimize_with_flags(const char* input, size_t len, uint64_t flags);

/**
 * Optimize an SVG string with an explicit plugin selection bitmask and extended options.
 * number_precision sets the decimal digit precision for the Number Precision plugin (clamped 1–10);
 * only used when ESVG_PLUGIN_NUMBER_PRECISION is set in flags.
 */
char* esvg_optimize_with_flags_ex(const char* input, size_t len, uint64_t flags, uint32_t number_precision);

/** Free a string returned by esvg_optimize or esvg_optimize_with_flags. NULL is safe. */
void esvg_free(char* ptr);

#ifdef __cplusplus
}
#endif
