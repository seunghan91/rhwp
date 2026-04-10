// rhwp iOS FFI — Swift Bridging Header
#ifndef RHWP_H
#define RHWP_H

#include <stdint.h>
#include <stddef.h>

typedef struct RhwpHandle RhwpHandle;

typedef struct {
    double width_pt;
    double height_pt;
} RhwpPageSize;

struct RhwpHandle *rhwp_open(const uint8_t *data, size_t len);
uint32_t rhwp_page_count(const struct RhwpHandle *handle);
RhwpPageSize rhwp_page_size(const struct RhwpHandle *handle, uint32_t page);
char *rhwp_render_page_svg(const struct RhwpHandle *handle, uint32_t page);
void rhwp_free_string(char *ptr);
void rhwp_close(struct RhwpHandle *handle);

#endif
