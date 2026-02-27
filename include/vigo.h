#ifndef VIGO_H
#define VIGO_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Maximum buffer size for heapless mode.
 */
#define MAX_BUFFER_SIZE 64

/**
 * Maximum output size for heapless mode.
 */
#define MAX_OUTPUT_SIZE 128

/**
 * Opaque handle to a vigo engine instance.
 */
typedef struct vigo_engine_t vigo_engine_t;

/**
 * Creates a new vigo engine with Telex input method.
 * Returns a pointer that must be freed with `vigo_free`.
 */
struct vigo_engine_t *vigo_new_telex(void);

/**
 * Creates a new vigo engine with VNI input method.
 * Returns a pointer that must be freed with `vigo_free`.
 */
struct vigo_engine_t *vigo_new_vni(void);

/**
 * Frees a vigo engine instance.
 */
void vigo_free(struct vigo_engine_t *engine);

/**
 * Feeds a character into the engine.
 * Returns true if the character was processed.
 */
bool vigo_feed(struct vigo_engine_t *engine, uint32_t ch);

/**
 * Gets the current output as a C string.
 * The returned string must be freed with `vigo_free_string`.
 */
char *vigo_get_output(const struct vigo_engine_t *engine);

/**
 * Gets the raw input as a C string.
 * The returned string must be freed with `vigo_free_string`.
 */
char *vigo_get_raw_input(const struct vigo_engine_t *engine);

/**
 * Frees a string returned by vigo functions.
 */
void vigo_free_string(char *s);

/**
 * Processes a backspace.
 * Returns true if a character was removed.
 */
bool vigo_backspace(struct vigo_engine_t *engine);

/**
 * Clears all input.
 */
void vigo_clear(struct vigo_engine_t *engine);

/**
 * Commits and returns the output, clearing the engine.
 * The returned string must be freed with `vigo_free_string`.
 */
char *vigo_commit(struct vigo_engine_t *engine);

/**
 * Returns true if the engine buffer is empty.
 */
bool vigo_is_empty(const struct vigo_engine_t *engine);

#endif /* VIGO_H */
