#include <wayland-client.h>
#include "wayland-pointer-constraints-unstable-v1-client-protocol.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

struct zwp_pointer_constraints_v1 *pc = NULL;

static void registry_global(void *data, struct wl_registry *registry,
                            uint32_t name, const char *interface, uint32_t version)
{
    if (strcmp(interface, zwp_pointer_constraints_v1_interface.name) == 0) {
        pc = wl_registry_bind(registry, name, &zwp_pointer_constraints_v1_interface, 1);
        printf("[pointer_constraints] bound zwp_pointer_constraints_v1\n");
    }
}

static void registry_global_remove(void *data, struct wl_registry *registry,
                                   uint32_t name)
{
    (void)data; (void)registry; (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_global_remove
};

void init_pointer_constraints(void *display_ptr)
{
    struct wl_display *display = (struct wl_display *)display_ptr;

    if (!display) {
        fprintf(stderr, "init_pointer_constraints: display is NULL\n");
        return;
    }

    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    wl_display_roundtrip(display);

    if (!pc) {
        fprintf(stderr, "Failed to bind zwp_pointer_constraints_v1\n");
    }
}

void lock_pointer(void *surface_ptr, void *pointer_ptr)
{
    if (!pc) return;
    zwp_pointer_constraints_v1_lock_pointer(pc,
        (struct wl_surface *)surface_ptr,
        (struct wl_pointer *)pointer_ptr,
        NULL,
        ZWP_POINTER_CONSTRAINTS_V1_LIFETIME_PERSISTENT);
    printf("Pointer locked!\n");
}
