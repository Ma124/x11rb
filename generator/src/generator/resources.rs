use super::{CreateInfo, ResourceInfo};

pub(super) fn for_extension(extension: &str) -> &'static [ResourceInfo<'static>] {
    EXTENSION_RESOURCES
        .iter()
        .find(|(ext, _)| extension == *ext)
        .map(|(_, info)| *info)
        .unwrap_or(&[])
}

const EXTENSION_RESOURCES: [(&str, &[ResourceInfo<'_>]); 13] = [(
    "xproto",
    &[
        ResourceInfo {
            resource_name: "Pixmap",
            create_requests: &[
                CreateInfo {
                    request_name: "CreatePixmap",
                    created_argument: "pid",
                },
                 CreateInfo {
                     request_name: "composite:NameWindowPixmap",
                     created_argument: "pixmap",
                 },
                 CreateInfo {
                     request_name: "dri3:PixmapFromBuffer",
                     created_argument: "pixmap",
                 },
                 CreateInfo {
                     request_name: "dri3:PixmapFromBuffers",
                     created_argument: "pixmap",
                 },
                 CreateInfo {
                     request_name: "shm:CreatePixmap",
                     created_argument: "pixmap",
                 },
            ],
            free_request: "FreePixmap",
        },
        ResourceInfo {
            resource_name: "Window",
            create_requests: &[
                CreateInfo {
                    request_name: "CreateWindow",
                    created_argument: "wid",
                },
            ],
            free_request: "DestroyWindow",
        },
        ResourceInfo {
            resource_name: "Font",
            create_requests: &[
                CreateInfo {
                    request_name: "OpenFont",
                    created_argument: "fid",
                },
            ],
            free_request: "CloseFont",
        },
        ResourceInfo {
            resource_name: "Gcontext",
            create_requests: &[
                CreateInfo {
                    request_name: "CreateGC",
                    created_argument: "cid",
                },
            ],
            free_request: "FreeGC",
        },
        ResourceInfo {
            resource_name: "Colormap",
            create_requests: &[
                CreateInfo {
                    request_name: "CreateColormap",
                    created_argument: "mid",
                },
            ],
            free_request: "FreeColormap",
        },
        ResourceInfo {
            resource_name: "Cursor",
            create_requests: &[
                CreateInfo {
                    request_name: "CreateCursor",
                    created_argument: "cid",
                },
                CreateInfo {
                    request_name: "CreateGlyphCursor",
                    created_argument: "cid",
                },
                 CreateInfo {
                     request_name: "render:CreateCursor",
                     created_argument: "cid",
                 },
                 CreateInfo {
                     request_name: "render:CreateAnimCursor",
                     created_argument: "cid",
                 },
            ],
            free_request: "FreeCursor",
        },
    ],
    ),
    ("composite", &[
     /*
      * Resource is from another namespace, but needs a special free
     ResourceInfo {
         resource_name: "xproto:Window",
         create_requests: &[
             CreateInfo {
                 request_name: "GetOverlayWindow",
                 created_argument: "window",
             },
         ],
         free_request: "ReleaseOverlayWindow",
     },
     */
    ]),
    ("damage", &[
     ResourceInfo {
         resource_name: "Damage",
         create_requests: &[
             CreateInfo {
                 request_name: "Create",
                 created_argument: "damage",
             },
         ],
         free_request: "Destroy",
     },
    ]),
    ("dri2", &[
     /*
      * DRAWABLE more or less comes from xproto?!?
     ResourceInfo {
         resource_name: "DRAWABLE",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateDrawable",
                 created_argument: "drawable",
             },
         ],
         free_request: "DestroyDrawable",
     },
     */
    ]),
    ("glx", &[
     // There are lots of candidates, but this being GLX I doubt anyone will ever use this.
     // My guesses: CreateGLXPixmap, CreatePixmap, CreateNewContext, CreatePbuffer, CreateWindow
    ]),
    ("record", &[
     ResourceInfo {
         resource_name: "Context",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateContext",
                 created_argument: "context",
             },
         ],
         free_request: "FreeContext",
     },
    ]),
    ("render", &[
     ResourceInfo {
         resource_name: "Picture",
         create_requests: &[
             CreateInfo {
                 request_name: "CreatePicture",
                 created_argument: "pid",
             },
             CreateInfo {
                 request_name: "CreateSolidFill",
                 created_argument: "picture",
             },
             CreateInfo {
                 request_name: "CreateLinearGradient",
                 created_argument: "picture",
             },
             CreateInfo {
                 request_name: "CreateRadialGradient",
                 created_argument: "picture",
             },
             CreateInfo {
                 request_name: "CreateConicalGradient",
                 created_argument: "picture",
             },
         ],
         free_request: "FreePicture",
     },
     ResourceInfo {
         resource_name: "Glyphset",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateGlyphSet",
                 created_argument: "gsid",
             },
         ],
         free_request: "FreeGlyphSet",
     },
    ]),
    ("shm", &[
     ResourceInfo {
         resource_name: "Seg",
         create_requests: &[
             CreateInfo {
                 request_name: "Attach",
                 created_argument: "shmseg",
             },
             CreateInfo {
                 request_name: "AttachFd",
                 created_argument: "shmseg",
             },
             /*
              * FD stuff
             CreateInfo {
                 request_name: "CreateSegment",
                 created_argument: "shmseg",
             },
             */
         ],
         free_request: "Detach",
     },
    ]),
    ("sync", &[
     ResourceInfo {
         resource_name: "Counter",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateCounter",
                 created_argument: "id",
             },
         ],
         free_request: "DestroyCounter",
     },
     ResourceInfo {
         resource_name: "Alarm",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateAlarm",
                 created_argument: "id",
             },
         ],
         free_request: "DestroyAlarm",
     },
     ResourceInfo {
         resource_name: "Fence",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateFence",
                 created_argument: "fence",
             },
             CreateInfo {
                 request_name: "dri3:FenceFromFD",
                 created_argument: "fence",
             },
         ],
         free_request: "DestroyFence",
     },
    ]),
    ("xf86dri", &[
     /*
     ResourceInfo {
         resource_name: "Context fixme: just u32",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateContext",
                 created_argument: "context",
             },
         ],
         free_request: "DestroyContext",
     },
     ResourceInfo {
         resource_name: "Drawable fixme: just u32",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateDrawable",
                 created_argument: "drawable",
             },
         ],
         free_request: "DestroyDrawable",
     },
     */
    ]),
    ("xfixes", &[
     ResourceInfo {
         resource_name: "Region",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateRegion",
                 created_argument: "region",
             },
             CreateInfo {
                 request_name: "CreateRegionFromBitmap",
                 created_argument: "region",
             },
             CreateInfo {
                 request_name: "CreateRegionFromWindow",
                 created_argument: "region",
             },
             CreateInfo {
                 request_name: "CreateRegionFromGC",
                 created_argument: "region",
             },
             CreateInfo {
                 request_name: "CreateRegionFromPicture",
                 created_argument: "region",
             },
             CreateInfo {
                 request_name: "composite:CreateRegionFromBorderClip",
                 created_argument: "region",
             },
         ],
         free_request: "DestroyRegion",
     },
    ]),
    ("xprint", &[
     /*
     ResourceInfo {
         resource_name: "Context todo actually u32",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateContext",
                 created_argument: "todo",
             },
         ],
         free_request: "todo",
     },
     */
    ]),
    ("xvmc", &[
     /* These requests have replies
     ResourceInfo {
         resource_name: "Context",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateContext",
                 created_argument: "context_id",
             },
         ],
         free_request: "DestroyContext",
     },
     ResourceInfo {
         resource_name: "Surface",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateSurface",
                 created_argument: "surface_id",
             },
         ],
         free_request: "DestroySurface",
     },
     ResourceInfo {
         resource_name: "Subpicture",
         create_requests: &[
             CreateInfo {
                 request_name: "CreateSubpicture",
                 created_argument: "subpicture_id",
             },
         ],
         free_request: "DestroySubpicture",
     },
     */
    ]),
    ];
