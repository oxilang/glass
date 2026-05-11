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

typedef struct {
  int error_code;
  char *payload;
} GlassResult;

GlassResult *glass_parse(const char *input);
GlassResult glass_serialize(const GlassValue *value);

GlassValueKind glass_value_get_kind(const GlassValue *ptr);
int glass_value_get_bool(const GlassValue *ptr);
double glass_value_get_number(const GlassValue *ptr);
const char *glass_value_get_string(const GlassValue *ptr);
const GlassArray *glass_value_get_array(const GlassValue *ptr);
const GlassMap *glass_value_get_map(const GlassValue *ptr);

size_t glass_array_len(const GlassArray *arr);
const GlassValue *glass_array_get(const GlassArray *arr, size_t index);

size_t glass_map_len(const GlassMap *map);
const GlassMapEntry *glass_map_get(const GlassMap *map, size_t index);
const char *glass_map_entry_key(const GlassMapEntry *entry);
const GlassValue *glass_map_entry_value(const GlassMapEntry *entry);

const char *glass_result_error_message(const GlassResult *res);
const GlassValue *glass_result_value(const GlassResult *res);
void glass_result_free(GlassResult *res);

#ifdef __cplusplus
}
#endif

#endif /* GLASS_H */
