/*
 * Vigo - Vietnamese Input Method Engine
 * C API Header
 */

#ifndef VIGO_H
#define VIGO_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to vigo engine */
typedef struct VigoEngine vigo_engine_t;

/*
 * Create a new vigo engine with Telex input method.
 * Returns a pointer that must be freed with vigo_free().
 */
vigo_engine_t* vigo_new_telex(void);

/*
 * Create a new vigo engine with VNI input method.
 * Returns a pointer that must be freed with vigo_free().
 */
vigo_engine_t* vigo_new_vni(void);

/*
 * Free a vigo engine instance.
 */
void vigo_free(vigo_engine_t* engine);

/*
 * Feed a character (Unicode codepoint) into the engine.
 * Returns true if the character was processed.
 */
bool vigo_feed(vigo_engine_t* engine, uint32_t ch);

/*
 * Get the current transformed output as a UTF-8 string.
 * The returned string must be freed with vigo_free_string().
 */
char* vigo_get_output(const vigo_engine_t* engine);

/*
 * Get the raw input buffer as a UTF-8 string.
 * The returned string must be freed with vigo_free_string().
 */
char* vigo_get_raw_input(const vigo_engine_t* engine);

/*
 * Free a string returned by vigo functions.
 */
void vigo_free_string(char* s);

/*
 * Process a backspace key.
 * Returns true if a character was removed.
 */
bool vigo_backspace(vigo_engine_t* engine);

/*
 * Clear all input.
 */
void vigo_clear(vigo_engine_t* engine);

/*
 * Commit the current output and clear the engine.
 * Returns the output string which must be freed with vigo_free_string().
 */
char* vigo_commit(vigo_engine_t* engine);

/*
 * Check if the engine buffer is empty.
 */
bool vigo_is_empty(const vigo_engine_t* engine);

#ifdef __cplusplus
}
#endif

#endif /* VIGO_H */
