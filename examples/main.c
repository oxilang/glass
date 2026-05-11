#include "glass.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void print_value(const GlassValue *value, int indent);

void print_indent(int indent) {
  for (int i = 0; i < indent; i++)
    printf("  ");
}

void print_string(const char *str) { printf("\"%s\"", str); }

void print_value(const GlassValue *value, int indent) {
  if (!value) {
    printf("null");
    return;
  }

  switch (glass_value_get_kind(value)) {
  case GLASS_NULL:
    printf("null");
    break;
  case GLASS_BOOL:
    printf("%s", glass_value_get_bool(value) ? "true" : "false");
    break;
  case GLASS_NUMBER:
    printf("%g", glass_value_get_number(value));
    break;
  case GLASS_STRING:
    print_string(glass_value_get_string(value));
    break;
  case GLASS_ARRAY: {
    const GlassArray *arr = glass_value_get_array(value);
    printf("[\n");
    for (size_t i = 0; i < glass_array_len(arr); i++) {
      print_indent(indent + 1);
      print_value(glass_array_get(arr, i), indent + 1);
      if (i < glass_array_len(arr) - 1)
        printf(",");
      printf("\n");
    }
    print_indent(indent);
    printf("]");
    break;
  }
  case GLASS_MAP: {
    const GlassMap *map = glass_value_get_map(value);
    printf("{\n");
    for (size_t i = 0; i < glass_map_len(map); i++) {
      const GlassMapEntry *entry = glass_map_get(map, i);
      print_indent(indent + 1);
      print_string(glass_map_entry_key(entry));
      printf(" ");
      print_value(glass_map_entry_value(entry), indent + 1);
      if (i < glass_map_len(map) - 1)
        printf(",");
      printf("\n");
    }
    print_indent(indent);
    printf("}");
    break;
  }
  }
}

int main(void) {
  const char *input = "root {\n"
                      "  name \"Alice\",\n"
                      "  age 30,\n"
                      "  hobbies [\n"
                      "    \"reading\",\n"
                      "    \"coding\",\n"
                      "  ],\n"
                      "  address {\n"
                      "    city \"Seattle\",\n"
                      "    zip \"98101\",\n"
                      "  },\n"
                      "},";

  printf("=== Parsing glass input ===\n");
  printf("%s\n\n", input);

  GlassResult *result = glass_parse(input);

  if (result->error_code != 0) {
    printf("Parse error: %s\n", glass_result_error_message(result));
    glass_result_free(result);
    return 1;
  }

  const GlassValue *value = glass_result_value(result);

  printf("=== Parsed value ===\n");
  print_value(value, 0);
  printf("\n\n");

  printf("=== Serializing back ===\n");
  GlassResult *ser_result = glass_serialize(value);
  if (ser_result->error_code != 0) {
    printf("Serialize error: %s\n", glass_result_error_message(ser_result));
  } else {
    printf("%s\n", glass_result_serialized(ser_result));
  }

  glass_result_free(ser_result);
  glass_result_free(result);

  return 0;
}
