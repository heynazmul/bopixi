// C ABI of the bopixiblitz Rust static library (ios/BlitzKit/rust).
// One BopixiBlitzDoc per frame. Drive every call for a document from one serial
// queue. Coordinates are CSS pixels in the frame's own space.
#ifndef BOPIXIBLITZ_H
#define BOPIXIBLITZ_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct BopixiBlitzDoc BopixiBlitzDoc;
typedef void (*BopixiBlitzWakeCallback)(size_t doc_id);

/// Called from a network thread when a pending font or image for `doc_id` arrives.
void bopixiblitz_set_wake_callback(BopixiBlitzWakeCallback callback);

BopixiBlitzDoc *bopixiblitz_doc_new(const char *html, const char *base_url, uint32_t width, uint32_t height, float scale);
void bopixiblitz_doc_free(BopixiBlitzDoc *doc);
size_t bopixiblitz_doc_id(const BopixiBlitzDoc *doc);
void bopixiblitz_doc_set_html(BopixiBlitzDoc *doc, const char *html);
void bopixiblitz_doc_set_viewport(BopixiBlitzDoc *doc, uint32_t width, uint32_t height, float scale);
bool bopixiblitz_doc_loading(const BopixiBlitzDoc *doc);

/// RGBA8, premultiplied alpha over white. Free with bopixiblitz_buffer_free.
uint8_t *bopixiblitz_doc_render(BopixiBlitzDoc *doc, uint32_t *out_width, uint32_t *out_height, size_t *out_len);
/// Same, with CSS animations advanced to `time` seconds.
uint8_t *bopixiblitz_doc_render_at(BopixiBlitzDoc *doc, double time, uint32_t *out_width, uint32_t *out_height, size_t *out_len);
/// True when the document has running CSS animations or transitions.
bool bopixiblitz_doc_is_animating(const BopixiBlitzDoc *doc);
/// Paint only the frame region (x, y, width, height in CSS px) at `scale` bitmap px per CSS px.
uint8_t *bopixiblitz_doc_render_region(BopixiBlitzDoc *doc, float scale, float x, float y, float width, float height,
                                     uint32_t *out_width, uint32_t *out_height, size_t *out_len);
/// Same, with CSS animations advanced to `time` seconds.
uint8_t *bopixiblitz_doc_render_region_at(BopixiBlitzDoc *doc, double time, float scale, float x, float y, float width,
                                        float height, uint32_t *out_width, uint32_t *out_height, size_t *out_len);
void bopixiblitz_buffer_free(uint8_t *ptr, size_t len);

/// Node ids are opaque non-zero values; 0 means none.
uint64_t bopixiblitz_doc_element_from_point(BopixiBlitzDoc *doc, float x, float y);
uint64_t bopixiblitz_doc_query_selector(const BopixiBlitzDoc *doc, const char *selector);
uint64_t bopixiblitz_node_parent(const BopixiBlitzDoc *doc, uint64_t node);
/// out receives x, y, width, height in CSS pixels.
bool bopixiblitz_node_rect(const BopixiBlitzDoc *doc, uint64_t node, float *out);
/// JSON: {id, tag, idAttr, classes, selector, text, parent, rect}. Free with bopixiblitz_string_free.
char *bopixiblitz_node_info(const BopixiBlitzDoc *doc, uint64_t node);
void bopixiblitz_string_free(char *ptr);

#ifdef __cplusplus
}
#endif
#endif
