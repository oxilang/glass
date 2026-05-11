#ifndef GLASS_H
#define GLASS_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  GLASS_NULL = 0,
  GLASS_BOOL = 1,
  GLASS_NUMBER = 2,
  GLASS_STRING = 3,
  GLASS_ARRAY = 4,
  GLASS_MAP = 5
} GlassValueKind;

typedef struct {
  GlassValueKind kind;
  union {
    bool bool_val;
    double number_val;
    char *string_val;
    struct GlassArray *array_val;
    struct GlassMap *map_val;
  } data;
} GlassValue;

typedef struct {
  size_t len;
  GlassValue *data;
} GlassArray;

typedef struct {
  char *key;
  GlassValue value;
} GlassMapEntry;

typedef struct {
  size_t len;
  GlassMapEntry *entries;
} GlassMap;

typedef enum {
  GLASS_RESULT_PARSE_SUCCESS = 0,
  GLASS_RESULT_SERIALIZE_SUCCESS = 1,
  GLASS_RESULT_ERROR = 2,
} GlassResultKind;

typedef union {
  GlassValue *value;
  char *serialized;
  char *error_message;
} GlassResultPayload;

typedef struct {
  int error_code;
  GlassResultKind kind;
  GlassResultPayload payload;
} GlassResult;

/* Parse a glass string. Returns a GlassResult that must be freed with
   glass_result_free. input may be NULL (returns error result). If non-NULL,
   must be null-terminated. */
GlassResult *glass_parse(const char *input);
/* Serialize a GlassValue to glass. Returns a GlassResult that must be freed
   with glass_result_free. value may be NULL (returns error result). If
   non-NULL, must point to a valid GlassValue. */
GlassResult *glass_serialize(const GlassValue *value);

/* ptr must be non-NULL and point to a valid GlassValue. */
GlassValueKind glass_value_get_kind(const GlassValue *ptr);
/* ptr must be non-NULL, valid, and kind must be GLASS_BOOL. */
int glass_value_get_bool(const GlassValue *ptr);
/* ptr must be non-NULL, valid, and kind must be GLASS_NUMBER. */
double glass_value_get_number(const GlassValue *ptr);
/* ptr must be non-NULL, valid, and kind must be GLASS_STRING.
   Returned pointer is valid until the owning GlassResult is freed. */
const char *glass_value_get_string(const GlassValue *ptr);
/* ptr must be non-NULL, valid, and kind must be GLASS_ARRAY. */
const GlassArray *glass_value_get_array(const GlassValue *ptr);
/* ptr must be non-NULL, valid, and kind must be GLASS_MAP. */
const GlassMap *glass_value_get_map(const GlassValue *ptr);

/* arr must be non-NULL and point to a valid GlassArray. */
size_t glass_array_len(const GlassArray *arr);
/* arr must be non-NULL and valid; index must be < arr->len. */
const GlassValue *glass_array_get(const GlassArray *arr, size_t index);

/* map must be non-NULL and point to a valid GlassMap. */
size_t glass_map_len(const GlassMap *map);
/* map must be non-NULL and valid; index must be < map->len. */
const GlassMapEntry *glass_map_get(const GlassMap *map, size_t index);
/* entry must be non-NULL and point to a valid GlassMapEntry. */
const char *glass_map_entry_key(const GlassMapEntry *entry);
/* entry must be non-NULL and point to a valid GlassMapEntry. */
const GlassValue *glass_map_entry_value(const GlassMapEntry *entry);

/* res must be non-NULL and valid. Returned pointer is valid until
 * glass_result_free. Returns NULL if kind is not GLASS_RESULT_ERROR. */
const char *glass_result_error_message(const GlassResult *res);
/* res must be non-NULL and valid. Returns NULL if kind is not
   GLASS_RESULT_PARSE_SUCCESS. When non-NULL, returned pointer is valid
   until glass_result_free. */
const GlassValue *glass_result_value(const GlassResult *res);
/* res must be non-NULL and valid. Returned pointer is valid until
 * glass_result_free. Returns NULL if kind is not
 * GLASS_RESULT_SERIALIZE_SUCCESS. */
const char *glass_result_serialized(const GlassResult *res);
/* res must be non-NULL and valid. Returns the kind of the result. */
GlassResultKind glass_result_get_kind(const GlassResult *res);
/* Frees a GlassResult previously returned by glass_parse or glass_serialize.
   res may be NULL (no-op). After this call, the pointer is invalidated. */
void glass_result_free(GlassResult *res);

#ifdef __cplusplus
}
#endif

#endif /* GLASS_H */
