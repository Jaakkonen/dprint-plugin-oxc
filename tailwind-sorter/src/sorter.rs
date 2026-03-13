// Copyright 2025 Anomaly Security Inc.
// Author: Anomaly Labs
//
// Licensed under the MIT License (the "License");
// you may not use this file except in compliance with the License.

//! Property-based Tailwind CSS v4 class sorter.
//!
//! Implements the same sorting algorithm as Tailwind CSS v4's compiler
//! (see `tailwindcss/packages/tailwindcss/src/compile.ts`, `getPropertySort`):
//!
//! 1. Each utility class maps to one or more CSS properties.
//! 2. Each CSS property has an index in `PROPERTY_ORDER` (from `property-order.ts`).
//! 3. The sort key is the sorted, deduplicated set of property indices.
//! 4. Classes are compared by: property indices (element-by-element, ascending),
//!    then by count (more properties → earlier), then alphabetically.
//! 5. Variant prefixes (e.g. `hover:`, `md:`) are stripped to find the base
//!    utility; classes with variants sort after identical base classes.
//! 6. Unknown classes (not matching any prefix) sort last, preserving relative order.

use once_cell::sync::Lazy;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// CSS Property Order — from tailwindcss/packages/tailwindcss/src/property-order.ts
// This is the canonical sort order for Tailwind CSS v4.
// Index in this array = sort priority (lower = earlier).
// ---------------------------------------------------------------------------

static PROPERTY_ORDER: &[&str] = &[
    "container-type",
    "pointer-events",
    "visibility",
    "position",
    "inset",
    "inset-inline",
    "inset-block",
    "inset-inline-start",
    "inset-inline-end",
    "top",
    "right",
    "bottom",
    "left",
    "isolation",
    "z-index",
    "order",
    "grid-column",
    "grid-column-start",
    "grid-column-end",
    "grid-row",
    "grid-row-start",
    "grid-row-end",
    "float",
    "clear",
    "--tw-container-component",
    "margin",
    "margin-inline",
    "margin-block",
    "margin-inline-start",
    "margin-inline-end",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "box-sizing",
    "display",
    "field-sizing",
    "aspect-ratio",
    "height",
    "max-height",
    "min-height",
    "width",
    "max-width",
    "min-width",
    "flex",
    "flex-shrink",
    "flex-grow",
    "flex-basis",
    "table-layout",
    "caption-side",
    "border-collapse",
    "border-spacing",
    "transform-origin",
    "translate",
    "--tw-translate-x",
    "--tw-translate-y",
    "--tw-translate-z",
    "scale",
    "--tw-scale-x",
    "--tw-scale-y",
    "--tw-scale-z",
    "rotate",
    "--tw-rotate-x",
    "--tw-rotate-y",
    "--tw-rotate-z",
    "--tw-skew-x",
    "--tw-skew-y",
    "transform",
    "animation",
    "cursor",
    "touch-action",
    "--tw-pan-x",
    "--tw-pan-y",
    "--tw-pinch-zoom",
    "resize",
    "scroll-snap-type",
    "--tw-scroll-snap-strictness",
    "scroll-snap-align",
    "scroll-snap-stop",
    "scroll-margin",
    "scroll-margin-inline",
    "scroll-margin-block",
    "scroll-margin-inline-start",
    "scroll-margin-inline-end",
    "scroll-margin-top",
    "scroll-margin-right",
    "scroll-margin-bottom",
    "scroll-margin-left",
    "scroll-padding",
    "scroll-padding-inline",
    "scroll-padding-block",
    "scroll-padding-inline-start",
    "scroll-padding-inline-end",
    "scroll-padding-top",
    "scroll-padding-right",
    "scroll-padding-bottom",
    "scroll-padding-left",
    "list-style-position",
    "list-style-type",
    "list-style-image",
    "appearance",
    "columns",
    "break-before",
    "break-inside",
    "break-after",
    "grid-auto-columns",
    "grid-auto-flow",
    "grid-auto-rows",
    "grid-template-columns",
    "grid-template-rows",
    "flex-direction",
    "flex-wrap",
    "place-content",
    "place-items",
    "align-content",
    "align-items",
    "justify-content",
    "justify-items",
    "gap",
    "column-gap",
    "row-gap",
    "--tw-space-x-reverse",
    "--tw-space-y-reverse",
    "divide-x-width",
    "divide-y-width",
    "--tw-divide-y-reverse",
    "divide-style",
    "divide-color",
    "place-self",
    "align-self",
    "justify-self",
    "overflow",
    "overflow-x",
    "overflow-y",
    "overscroll-behavior",
    "overscroll-behavior-x",
    "overscroll-behavior-y",
    "scroll-behavior",
    "border-radius",
    "border-start-radius",
    "border-end-radius",
    "border-top-radius",
    "border-right-radius",
    "border-bottom-radius",
    "border-left-radius",
    "border-start-start-radius",
    "border-start-end-radius",
    "border-end-end-radius",
    "border-end-start-radius",
    "border-top-left-radius",
    "border-top-right-radius",
    "border-bottom-right-radius",
    "border-bottom-left-radius",
    "border-width",
    "border-inline-width",
    "border-block-width",
    "border-inline-start-width",
    "border-inline-end-width",
    "border-top-width",
    "border-right-width",
    "border-bottom-width",
    "border-left-width",
    "border-style",
    "border-inline-style",
    "border-block-style",
    "border-inline-start-style",
    "border-inline-end-style",
    "border-top-style",
    "border-right-style",
    "border-bottom-style",
    "border-left-style",
    "border-color",
    "border-inline-color",
    "border-block-color",
    "border-inline-start-color",
    "border-inline-end-color",
    "border-top-color",
    "border-right-color",
    "border-bottom-color",
    "border-left-color",
    "background-color",
    "background-image",
    "--tw-gradient-position",
    "--tw-gradient-stops",
    "--tw-gradient-via-stops",
    "--tw-gradient-from",
    "--tw-gradient-from-position",
    "--tw-gradient-via",
    "--tw-gradient-via-position",
    "--tw-gradient-to",
    "--tw-gradient-to-position",
    "mask-image",
    "--tw-mask-top",
    "--tw-mask-top-from-color",
    "--tw-mask-top-from-position",
    "--tw-mask-top-to-color",
    "--tw-mask-top-to-position",
    "--tw-mask-right",
    "--tw-mask-right-from-color",
    "--tw-mask-right-from-position",
    "--tw-mask-right-to-color",
    "--tw-mask-right-to-position",
    "--tw-mask-bottom",
    "--tw-mask-bottom-from-color",
    "--tw-mask-bottom-from-position",
    "--tw-mask-bottom-to-color",
    "--tw-mask-bottom-to-position",
    "--tw-mask-left",
    "--tw-mask-left-from-color",
    "--tw-mask-left-from-position",
    "--tw-mask-left-to-color",
    "--tw-mask-left-to-position",
    "--tw-mask-linear",
    "--tw-mask-linear-position",
    "--tw-mask-linear-from-color",
    "--tw-mask-linear-from-position",
    "--tw-mask-linear-to-color",
    "--tw-mask-linear-to-position",
    "--tw-mask-radial",
    "--tw-mask-radial-shape",
    "--tw-mask-radial-size",
    "--tw-mask-radial-position",
    "--tw-mask-radial-from-color",
    "--tw-mask-radial-from-position",
    "--tw-mask-radial-to-color",
    "--tw-mask-radial-to-position",
    "--tw-mask-conic",
    "--tw-mask-conic-position",
    "--tw-mask-conic-from-color",
    "--tw-mask-conic-from-position",
    "--tw-mask-conic-to-color",
    "--tw-mask-conic-to-position",
    "box-decoration-break",
    "background-size",
    "background-attachment",
    "background-clip",
    "background-position",
    "background-repeat",
    "background-origin",
    "mask-composite",
    "mask-mode",
    "mask-type",
    "mask-size",
    "mask-clip",
    "mask-position",
    "mask-repeat",
    "mask-origin",
    "fill",
    "stroke",
    "stroke-width",
    "object-fit",
    "object-position",
    "padding",
    "padding-inline",
    "padding-block",
    "padding-inline-start",
    "padding-inline-end",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "text-align",
    "text-indent",
    "vertical-align",
    "font-family",
    "font-size",
    "line-height",
    "font-weight",
    "letter-spacing",
    "text-wrap",
    "overflow-wrap",
    "word-break",
    "text-overflow",
    "hyphens",
    "white-space",
    "color",
    "text-transform",
    "font-style",
    "font-stretch",
    "font-variant-numeric",
    "text-decoration-line",
    "text-decoration-color",
    "text-decoration-style",
    "text-decoration-thickness",
    "text-underline-offset",
    "-webkit-font-smoothing",
    "placeholder-color",
    "caret-color",
    "accent-color",
    "color-scheme",
    "opacity",
    "background-blend-mode",
    "mix-blend-mode",
    "box-shadow",
    "--tw-shadow",
    "--tw-shadow-color",
    "--tw-ring-shadow",
    "--tw-ring-color",
    "--tw-inset-shadow",
    "--tw-inset-shadow-color",
    "--tw-inset-ring-shadow",
    "--tw-inset-ring-color",
    "--tw-ring-offset-width",
    "--tw-ring-offset-color",
    "outline",
    "outline-width",
    "outline-offset",
    "outline-color",
    "--tw-blur",
    "--tw-brightness",
    "--tw-contrast",
    "--tw-drop-shadow",
    "--tw-grayscale",
    "--tw-hue-rotate",
    "--tw-invert",
    "--tw-saturate",
    "--tw-sepia",
    "filter",
    "--tw-backdrop-blur",
    "--tw-backdrop-brightness",
    "--tw-backdrop-contrast",
    "--tw-backdrop-grayscale",
    "--tw-backdrop-hue-rotate",
    "--tw-backdrop-invert",
    "--tw-backdrop-opacity",
    "--tw-backdrop-saturate",
    "--tw-backdrop-sepia",
    "backdrop-filter",
    "transition-property",
    "transition-behavior",
    "transition-delay",
    "transition-duration",
    "transition-timing-function",
    "will-change",
    "contain",
    "content",
    "forced-color-adjust",
];

// ---------------------------------------------------------------------------
// Property index lookup — built once at startup
// ---------------------------------------------------------------------------

static PROPERTY_INDEX: Lazy<HashMap<&'static str, usize>> = Lazy::new(|| {
    PROPERTY_ORDER
        .iter()
        .enumerate()
        .map(|(i, &prop)| (prop, i))
        .collect()
});

// ---------------------------------------------------------------------------
// Utility class → CSS property mappings
//
// Each entry maps a class prefix to the CSS properties it generates.
// The sort key is derived from the property indices exactly as Tailwind's
// `getPropertySort` does in compile.ts.
//
// When a utility uses `--tw-sort`, the sort value replaces the normal
// property lookup (if that value exists in PROPERTY_ORDER).
//
// Entries are ordered roughly by their position in property-order.ts so
// the table is easy to cross-reference.
// ---------------------------------------------------------------------------

/// A mapping entry: (class_prefix, &[css_properties])
///
/// For functional utilities like `p-4`, the prefix is `"p"`.
/// For static utilities like `flex`, the prefix is `"flex"`.
/// For multi-word statics like `inline-block`, the prefix is `"inline-block"`.
///
/// The `css_properties` slice contains the CSS property names (or `--tw-*`
/// custom properties) that this utility generates. Only properties present
/// in `PROPERTY_ORDER` contribute to the sort key.
static CLASS_TO_PROPERTIES: &[(&str, &[&str])] = &[
    // -- Container queries (@container) --
    ("@container", &["container-type"]),

    // -- Pointer events --
    ("pointer-events-none", &["pointer-events"]),
    ("pointer-events-auto", &["pointer-events"]),

    // -- Visibility --
    ("visible", &["visibility"]),
    ("invisible", &["visibility"]),
    ("collapse", &["visibility"]),

    // -- Position --
    ("static", &["position"]),
    ("fixed", &["position"]),
    ("absolute", &["position"]),
    ("relative", &["position"]),
    ("sticky", &["position"]),

    // -- Inset --
    ("inset", &["inset"]),
    ("inset-x", &["inset-inline"]),
    ("inset-y", &["inset-block"]),
    ("start", &["inset-inline-start"]),
    ("end", &["inset-inline-end"]),
    ("top", &["top"]),
    ("right", &["right"]),
    ("bottom", &["bottom"]),
    ("left", &["left"]),

    // -- Isolation --
    ("isolate", &["isolation"]),
    ("isolation-auto", &["isolation"]),

    // -- Z-index --
    ("z", &["z-index"]),

    // -- Order --
    ("order", &["order"]),

    // -- Grid column --
    ("col", &["grid-column"]),
    ("col-span", &["grid-column"]),
    ("col-start", &["grid-column-start"]),
    ("col-end", &["grid-column-end"]),

    // -- Grid row --
    ("row", &["grid-row"]),
    ("row-span", &["grid-row"]),
    ("row-start", &["grid-row-start"]),
    ("row-end", &["grid-row-end"]),

    // -- Float --
    ("float-start", &["float"]),
    ("float-end", &["float"]),
    ("float-right", &["float"]),
    ("float-left", &["float"]),
    ("float-none", &["float"]),

    // -- Clear --
    ("clear-start", &["clear"]),
    ("clear-end", &["clear"]),
    ("clear-right", &["clear"]),
    ("clear-left", &["clear"]),
    ("clear-both", &["clear"]),
    ("clear-none", &["clear"]),

    // -- Container (--tw-sort: --tw-container-component) --
    ("container", &["--tw-container-component"]),

    // -- Margin --
    ("m", &["margin"]),
    ("mx", &["margin-inline"]),
    ("my", &["margin-block"]),
    ("ms", &["margin-inline-start"]),
    ("me", &["margin-inline-end"]),
    ("mt", &["margin-top"]),
    ("mr", &["margin-right"]),
    ("mb", &["margin-bottom"]),
    ("ml", &["margin-left"]),

    // -- Box sizing --
    ("box-border", &["box-sizing"]),
    ("box-content", &["box-sizing"]),

    // -- Display --
    ("block", &["display"]),
    ("inline-block", &["display"]),
    ("inline", &["display"]),
    ("flex", &["display"]),  // Note: static `flex` = display:flex
    ("inline-flex", &["display"]),
    ("table", &["display"]),
    ("inline-table", &["display"]),
    ("table-caption", &["display"]),
    ("table-cell", &["display"]),
    ("table-column", &["display"]),
    ("table-column-group", &["display"]),
    ("table-footer-group", &["display"]),
    ("table-header-group", &["display"]),
    ("table-row-group", &["display"]),
    ("table-row", &["display"]),
    ("flow-root", &["display"]),
    ("grid", &["display"]),
    ("inline-grid", &["display"]),
    ("contents", &["display"]),
    ("list-item", &["display"]),
    ("hidden", &["display"]),

    // -- Line clamp (sorts by display — first recognized property) --
    ("line-clamp", &["display"]),

    // -- Field sizing --
    ("field-sizing-content", &["field-sizing"]),
    ("field-sizing-fixed", &["field-sizing"]),

    // -- Aspect ratio --
    ("aspect", &["aspect-ratio"]),

    // -- Size (--tw-sort: "size" — NOT in property-order, falls through to width+height) --
    ("size", &["width", "height"]),

    // -- Width / Height --
    ("h", &["height"]),
    ("max-h", &["max-height"]),
    ("min-h", &["min-height"]),
    ("w", &["width"]),
    ("max-w", &["max-width"]),
    ("min-w", &["min-width"]),

    // -- Flex (functional: flex-1, flex-auto, etc. — the CSS `flex` property) --
    ("flex-auto", &["flex"]),
    ("flex-initial", &["flex"]),
    ("flex-none", &["flex"]),
    // Numeric flex values like `flex-1` need the CSS `flex` property (not display).
    // We add explicit entries for common numeric values. The prefix fallback "flex"
    // still maps to display (for bare `flex` meaning `display: flex`).
    ("flex-1", &["flex"]),
    ("flex-2", &["flex"]),
    ("flex-3", &["flex"]),
    ("flex-4", &["flex"]),
    ("flex-5", &["flex"]),
    ("flex-6", &["flex"]),
    ("flex-7", &["flex"]),
    ("flex-8", &["flex"]),
    ("flex-9", &["flex"]),
    ("flex-10", &["flex"]),
    ("flex-11", &["flex"]),
    ("flex-12", &["flex"]),

    // -- Flex shrink / grow --
    ("shrink", &["flex-shrink"]),
    ("grow", &["flex-grow"]),

    // -- Flex basis --
    ("basis", &["flex-basis"]),

    // -- Table layout --
    ("table-auto", &["table-layout"]),
    ("table-fixed", &["table-layout"]),

    // -- Caption side --
    ("caption-top", &["caption-side"]),
    ("caption-bottom", &["caption-side"]),

    // -- Border collapse --
    ("border-collapse", &["border-collapse"]),
    ("border-separate", &["border-collapse"]),

    // -- Border spacing --
    ("border-spacing", &["border-spacing"]),
    ("border-spacing-x", &["border-spacing"]),
    ("border-spacing-y", &["border-spacing"]),

    // -- Transform origin --
    ("origin", &["transform-origin"]),

    // -- Translate --
    ("translate", &["translate"]),
    ("translate-x", &["--tw-translate-x", "translate"]),
    ("translate-y", &["--tw-translate-y", "translate"]),
    ("translate-z", &["--tw-translate-z", "translate"]),
    ("translate-3d", &["translate"]),

    // -- Scale --
    ("scale", &["scale"]),
    ("scale-x", &["--tw-scale-x", "scale"]),
    ("scale-y", &["--tw-scale-y", "scale"]),
    ("scale-z", &["--tw-scale-z", "scale"]),
    ("scale-3d", &["scale"]),
    ("scale-none", &["scale"]),

    // -- Rotate --
    ("rotate", &["rotate"]),
    ("rotate-none", &["rotate"]),
    ("rotate-x", &["--tw-rotate-x", "transform"]),
    ("rotate-y", &["--tw-rotate-y", "transform"]),
    ("rotate-z", &["--tw-rotate-z", "transform"]),

    // -- Skew --
    ("skew", &["--tw-skew-x", "--tw-skew-y", "transform"]),
    ("skew-x", &["--tw-skew-x", "transform"]),
    ("skew-y", &["--tw-skew-y", "transform"]),

    // -- Transform --
    ("transform-cpu", &["transform"]),
    ("transform-gpu", &["transform"]),
    ("transform-none", &["transform"]),
    ("transform-flat", &["transform"]),
    ("transform-3d", &["transform"]),

    // -- Animation --
    ("animate", &["animation"]),

    // -- Cursor --
    ("cursor", &["cursor"]),

    // -- Touch action --
    ("touch-auto", &["touch-action"]),
    ("touch-none", &["touch-action"]),
    ("touch-manipulation", &["touch-action"]),
    ("touch-pan-x", &["--tw-pan-x", "touch-action"]),
    ("touch-pan-left", &["--tw-pan-x", "touch-action"]),
    ("touch-pan-right", &["--tw-pan-x", "touch-action"]),
    ("touch-pan-y", &["--tw-pan-y", "touch-action"]),
    ("touch-pan-up", &["--tw-pan-y", "touch-action"]),
    ("touch-pan-down", &["--tw-pan-y", "touch-action"]),
    ("touch-pinch-zoom", &["--tw-pinch-zoom", "touch-action"]),

    // -- Resize --
    ("resize", &["resize"]),
    ("resize-none", &["resize"]),
    ("resize-x", &["resize"]),
    ("resize-y", &["resize"]),

    // -- Scroll snap type --
    ("snap-none", &["scroll-snap-type"]),
    ("snap-x", &["scroll-snap-type"]),
    ("snap-y", &["scroll-snap-type"]),
    ("snap-both", &["scroll-snap-type"]),
    ("snap-mandatory", &["--tw-scroll-snap-strictness"]),
    ("snap-proximity", &["--tw-scroll-snap-strictness"]),

    // -- Scroll snap align --
    ("snap-align-none", &["scroll-snap-align"]),
    ("snap-start", &["scroll-snap-align"]),
    ("snap-end", &["scroll-snap-align"]),
    ("snap-center", &["scroll-snap-align"]),

    // -- Scroll snap stop --
    ("snap-normal", &["scroll-snap-stop"]),
    ("snap-always", &["scroll-snap-stop"]),

    // -- Scroll margin --
    ("scroll-m", &["scroll-margin"]),
    ("scroll-mx", &["scroll-margin-inline"]),
    ("scroll-my", &["scroll-margin-block"]),
    ("scroll-ms", &["scroll-margin-inline-start"]),
    ("scroll-me", &["scroll-margin-inline-end"]),
    ("scroll-mt", &["scroll-margin-top"]),
    ("scroll-mr", &["scroll-margin-right"]),
    ("scroll-mb", &["scroll-margin-bottom"]),
    ("scroll-ml", &["scroll-margin-left"]),

    // -- Scroll padding --
    ("scroll-p", &["scroll-padding"]),
    ("scroll-px", &["scroll-padding-inline"]),
    ("scroll-py", &["scroll-padding-block"]),
    ("scroll-ps", &["scroll-padding-inline-start"]),
    ("scroll-pe", &["scroll-padding-inline-end"]),
    ("scroll-pt", &["scroll-padding-top"]),
    ("scroll-pr", &["scroll-padding-right"]),
    ("scroll-pb", &["scroll-padding-bottom"]),
    ("scroll-pl", &["scroll-padding-left"]),

    // -- List style --
    ("list-inside", &["list-style-position"]),
    ("list-outside", &["list-style-position"]),
    ("list", &["list-style-type"]),
    ("list-image", &["list-style-image"]),

    // -- Appearance --
    ("appearance-none", &["appearance"]),
    ("appearance-auto", &["appearance"]),

    // -- Columns --
    ("columns", &["columns"]),

    // -- Break --
    ("break-before", &["break-before"]),
    ("break-inside", &["break-inside"]),
    ("break-after", &["break-after"]),

    // -- Grid auto --
    ("auto-cols", &["grid-auto-columns"]),
    ("grid-flow-row", &["grid-auto-flow"]),
    ("grid-flow-col", &["grid-auto-flow"]),
    ("grid-flow-dense", &["grid-auto-flow"]),
    ("grid-flow-row-dense", &["grid-auto-flow"]),
    ("grid-flow-col-dense", &["grid-auto-flow"]),
    ("auto-rows", &["grid-auto-rows"]),

    // -- Grid template --
    ("grid-cols", &["grid-template-columns"]),
    ("grid-rows", &["grid-template-rows"]),

    // -- Flex direction --
    ("flex-row", &["flex-direction"]),
    ("flex-row-reverse", &["flex-direction"]),
    ("flex-col", &["flex-direction"]),
    ("flex-col-reverse", &["flex-direction"]),

    // -- Flex wrap --
    ("flex-wrap", &["flex-wrap"]),
    ("flex-nowrap", &["flex-wrap"]),
    ("flex-wrap-reverse", &["flex-wrap"]),

    // -- Place content --
    ("place-content", &["place-content"]),

    // -- Place items --
    ("place-items", &["place-items"]),

    // -- Align content --
    ("content-center", &["align-content"]),
    ("content-start", &["align-content"]),
    ("content-end", &["align-content"]),
    ("content-between", &["align-content"]),
    ("content-around", &["align-content"]),
    ("content-evenly", &["align-content"]),
    ("content-baseline", &["align-content"]),
    ("content-stretch", &["align-content"]),
    ("content-normal", &["align-content"]),

    // -- Align items --
    ("items-start", &["align-items"]),
    ("items-end", &["align-items"]),
    ("items-center", &["align-items"]),
    ("items-baseline", &["align-items"]),
    ("items-stretch", &["align-items"]),

    // -- Justify content --
    ("justify-start", &["justify-content"]),
    ("justify-end", &["justify-content"]),
    ("justify-center", &["justify-content"]),
    ("justify-between", &["justify-content"]),
    ("justify-around", &["justify-content"]),
    ("justify-evenly", &["justify-content"]),
    ("justify-stretch", &["justify-content"]),
    ("justify-normal", &["justify-content"]),
    ("justify-baseline", &["justify-content"]),

    // -- Justify items --
    ("justify-items-start", &["justify-items"]),
    ("justify-items-end", &["justify-items"]),
    ("justify-items-center", &["justify-items"]),
    ("justify-items-stretch", &["justify-items"]),
    ("justify-items-normal", &["justify-items"]),

    // -- Gap --
    ("gap", &["gap"]),
    ("gap-x", &["column-gap"]),
    ("gap-y", &["row-gap"]),

    // -- Space (--tw-sort overrides) --
    ("space-x", &["row-gap"]),
    ("space-y", &["column-gap"]),
    ("space-x-reverse", &["--tw-space-x-reverse"]),
    ("space-y-reverse", &["--tw-space-y-reverse"]),

    // -- Divide width (--tw-sort overrides) --
    ("divide-x", &["divide-x-width"]),
    ("divide-y", &["divide-y-width"]),
    ("divide-x-reverse", &["--tw-divide-y-reverse"]),
    ("divide-y-reverse", &["--tw-divide-y-reverse"]),

    // -- Divide style (--tw-sort: divide-style) --
    ("divide-solid", &["divide-style"]),
    ("divide-dashed", &["divide-style"]),
    ("divide-dotted", &["divide-style"]),
    ("divide-double", &["divide-style"]),
    ("divide-none", &["divide-style"]),

    // -- Divide color (--tw-sort: divide-color) --
    // `divide-<color>` maps to divide-color. Disambiguation via DISAMBIG_COLOR_KEYS.
    ("divide", &["divide-color"]),

    // -- Place self --
    ("place-self-auto", &["place-self"]),
    ("place-self-start", &["place-self"]),
    ("place-self-end", &["place-self"]),
    ("place-self-center", &["place-self"]),
    ("place-self-stretch", &["place-self"]),
    ("place-self-baseline", &["place-self"]),

    // -- Align self --
    ("self-auto", &["align-self"]),
    ("self-start", &["align-self"]),
    ("self-end", &["align-self"]),
    ("self-center", &["align-self"]),
    ("self-stretch", &["align-self"]),
    ("self-baseline", &["align-self"]),

    // -- Justify self --
    ("justify-self-auto", &["justify-self"]),
    ("justify-self-start", &["justify-self"]),
    ("justify-self-end", &["justify-self"]),
    ("justify-self-center", &["justify-self"]),
    ("justify-self-stretch", &["justify-self"]),

    // -- Overflow --
    ("overflow-auto", &["overflow"]),
    ("overflow-hidden", &["overflow"]),
    ("overflow-clip", &["overflow"]),
    ("overflow-visible", &["overflow"]),
    ("overflow-scroll", &["overflow"]),
    ("overflow-x-auto", &["overflow-x"]),
    ("overflow-x-hidden", &["overflow-x"]),
    ("overflow-x-clip", &["overflow-x"]),
    ("overflow-x-visible", &["overflow-x"]),
    ("overflow-x-scroll", &["overflow-x"]),
    ("overflow-y-auto", &["overflow-y"]),
    ("overflow-y-hidden", &["overflow-y"]),
    ("overflow-y-clip", &["overflow-y"]),
    ("overflow-y-visible", &["overflow-y"]),
    ("overflow-y-scroll", &["overflow-y"]),

    // -- Overscroll --
    ("overscroll-auto", &["overscroll-behavior"]),
    ("overscroll-contain", &["overscroll-behavior"]),
    ("overscroll-none", &["overscroll-behavior"]),
    ("overscroll-x-auto", &["overscroll-behavior-x"]),
    ("overscroll-x-contain", &["overscroll-behavior-x"]),
    ("overscroll-x-none", &["overscroll-behavior-x"]),
    ("overscroll-y-auto", &["overscroll-behavior-y"]),
    ("overscroll-y-contain", &["overscroll-behavior-y"]),
    ("overscroll-y-none", &["overscroll-behavior-y"]),

    // -- Scroll behavior --
    ("scroll-auto", &["scroll-behavior"]),
    ("scroll-smooth", &["scroll-behavior"]),

    // -- Truncate (sorts by first recognized: overflow) --
    ("truncate", &["overflow"]),

    // -- Border radius --
    ("rounded", &["border-radius"]),
    ("rounded-s", &["border-start-start-radius", "border-end-start-radius"]),
    ("rounded-e", &["border-start-end-radius", "border-end-end-radius"]),
    ("rounded-t", &["border-top-left-radius", "border-top-right-radius"]),
    ("rounded-r", &["border-top-right-radius", "border-bottom-right-radius"]),
    ("rounded-b", &["border-bottom-right-radius", "border-bottom-left-radius"]),
    ("rounded-l", &["border-top-left-radius", "border-bottom-left-radius"]),
    ("rounded-ss", &["border-start-start-radius"]),
    ("rounded-se", &["border-start-end-radius"]),
    ("rounded-ee", &["border-end-end-radius"]),
    ("rounded-es", &["border-end-start-radius"]),
    ("rounded-tl", &["border-top-left-radius"]),
    ("rounded-tr", &["border-top-right-radius"]),
    ("rounded-br", &["border-bottom-right-radius"]),
    ("rounded-bl", &["border-bottom-left-radius"]),

    // -- Border width (specific directional entries — always width) --
    ("border-x", &["border-inline-width"]),
    ("border-y", &["border-block-width"]),
    ("border-s", &["border-inline-start-width"]),
    ("border-e", &["border-inline-end-width"]),
    ("border-t", &["border-top-width"]),
    ("border-r", &["border-right-width"]),
    ("border-b", &["border-bottom-width"]),
    ("border-l", &["border-left-width"]),

    // -- Border (bare `border` or `border-<number>` = width, `border-<color>` = color) --
    // Bare `border` = border-width: 1px. Specific numeric widths are explicit entries.
    // `border-<color>` disambiguation is handled in lookup_sort_key via DISAMBIG_COLOR_KEYS.
    ("border", &["border-width"]),
    ("border-0", &["border-width"]),
    ("border-2", &["border-width"]),
    ("border-4", &["border-width"]),
    ("border-8", &["border-width"]),

    // -- Border style --
    ("border-solid", &["border-style"]),
    ("border-dashed", &["border-style"]),
    ("border-dotted", &["border-style"]),
    ("border-double", &["border-style"]),
    ("border-hidden", &["border-style"]),
    ("border-none", &["border-style"]),

    // -- Background color --
    ("bg", &["background-color"]),

    // -- Background image / gradient --
    ("bg-none", &["background-image"]),
    ("bg-linear", &["background-image"]),
    ("bg-conic", &["background-image"]),
    ("bg-radial", &["background-image"]),

    // -- Gradient stops (--tw-sort overrides) --
    ("from", &["--tw-gradient-from"]),
    ("via", &["--tw-gradient-via"]),
    ("via-none", &["--tw-gradient-via-stops"]),
    ("to", &["--tw-gradient-to"]),

    // -- Mask --
    ("mask-none", &["mask-image"]),
    ("mask", &["mask-image"]),
    ("mask-add", &["mask-composite"]),
    ("mask-subtract", &["mask-composite"]),
    ("mask-intersect", &["mask-composite"]),
    ("mask-exclude", &["mask-composite"]),
    ("mask-alpha", &["mask-mode"]),
    ("mask-luminance", &["mask-mode"]),
    ("mask-match", &["mask-mode"]),
    ("mask-type-alpha", &["mask-type"]),
    ("mask-type-luminance", &["mask-type"]),
    ("mask-auto", &["mask-size"]),
    ("mask-cover", &["mask-size"]),
    ("mask-contain", &["mask-size"]),
    ("mask-size", &["mask-size"]),
    ("mask-position", &["mask-position"]),
    ("mask-repeat", &["mask-repeat"]),
    ("mask-no-repeat", &["mask-repeat"]),
    ("mask-repeat-x", &["mask-repeat"]),
    ("mask-repeat-y", &["mask-repeat"]),
    ("mask-clip", &["mask-clip"]),
    ("mask-origin", &["mask-origin"]),

    // -- Box decoration break --
    ("box-decoration-slice", &["box-decoration-break"]),
    ("box-decoration-clone", &["box-decoration-break"]),

    // -- Background size --
    ("bg-auto", &["background-size"]),
    ("bg-cover", &["background-size"]),
    ("bg-contain", &["background-size"]),
    ("bg-size", &["background-size"]),

    // -- Background attachment --
    ("bg-fixed", &["background-attachment"]),
    ("bg-local", &["background-attachment"]),
    ("bg-scroll", &["background-attachment"]),

    // -- Background clip --
    ("bg-clip-text", &["background-clip"]),
    ("bg-clip-border", &["background-clip"]),
    ("bg-clip-padding", &["background-clip"]),
    ("bg-clip-content", &["background-clip"]),

    // -- Background position --
    ("bg-top", &["background-position"]),
    ("bg-bottom", &["background-position"]),
    ("bg-center", &["background-position"]),
    ("bg-left", &["background-position"]),
    ("bg-right", &["background-position"]),
    ("bg-top-left", &["background-position"]),
    ("bg-top-right", &["background-position"]),
    ("bg-bottom-left", &["background-position"]),
    ("bg-bottom-right", &["background-position"]),
    ("bg-position", &["background-position"]),

    // -- Background repeat --
    ("bg-repeat", &["background-repeat"]),
    ("bg-no-repeat", &["background-repeat"]),
    ("bg-repeat-x", &["background-repeat"]),
    ("bg-repeat-y", &["background-repeat"]),
    ("bg-repeat-round", &["background-repeat"]),
    ("bg-repeat-space", &["background-repeat"]),

    // -- Background origin --
    ("bg-origin-border", &["background-origin"]),
    ("bg-origin-padding", &["background-origin"]),
    ("bg-origin-content", &["background-origin"]),

    // -- Fill / Stroke --
    ("fill", &["fill"]),
    ("fill-none", &["fill"]),
    ("stroke", &["stroke"]),
    ("stroke-none", &["stroke"]),

    // -- Object fit --
    ("object-contain", &["object-fit"]),
    ("object-cover", &["object-fit"]),
    ("object-fill", &["object-fit"]),
    ("object-none", &["object-fit"]),
    ("object-scale-down", &["object-fit"]),
    ("object", &["object-position"]),

    // -- Padding --
    ("p", &["padding"]),
    ("px", &["padding-inline"]),
    ("py", &["padding-block"]),
    ("ps", &["padding-inline-start"]),
    ("pe", &["padding-inline-end"]),
    ("pt", &["padding-top"]),
    ("pr", &["padding-right"]),
    ("pb", &["padding-bottom"]),
    ("pl", &["padding-left"]),

    // -- Text align --
    ("text-left", &["text-align"]),
    ("text-center", &["text-align"]),
    ("text-right", &["text-align"]),
    ("text-justify", &["text-align"]),
    ("text-start", &["text-align"]),
    ("text-end", &["text-align"]),

    // -- Text indent --
    ("indent", &["text-indent"]),

    // -- Vertical align --
    ("align-baseline", &["vertical-align"]),
    ("align-top", &["vertical-align"]),
    ("align-middle", &["vertical-align"]),
    ("align-bottom", &["vertical-align"]),
    ("align-text-top", &["vertical-align"]),
    ("align-text-bottom", &["vertical-align"]),
    ("align-sub", &["vertical-align"]),
    ("align-super", &["vertical-align"]),
    ("align", &["vertical-align"]),

    // -- Font family (font-sans, font-serif, font-mono) --
    ("font-sans", &["font-family"]),
    ("font-serif", &["font-family"]),
    ("font-mono", &["font-family"]),

    // -- Font size (text-sm, text-lg, etc.) --
    // Note: `text-<size>` maps to font-size, `text-<color>` maps to color.
    // Disambiguation is handled in `lookup_sort_key` — fallback prefix "text" maps
    // to "color" because most text-* values are colors; the known sizes (xs, sm,
    // base, lg, xl, 2xl–9xl) are added as explicit entries below.
    ("text", &["color"]),
    // Explicit text-size entries (these override the "text" → color fallback):
    ("text-xs", &["font-size", "line-height"]),
    ("text-sm", &["font-size", "line-height"]),
    ("text-base", &["font-size", "line-height"]),
    ("text-lg", &["font-size", "line-height"]),
    ("text-xl", &["font-size", "line-height"]),
    ("text-2xl", &["font-size", "line-height"]),
    ("text-3xl", &["font-size", "line-height"]),
    ("text-4xl", &["font-size", "line-height"]),
    ("text-5xl", &["font-size", "line-height"]),
    ("text-6xl", &["font-size", "line-height"]),
    ("text-7xl", &["font-size", "line-height"]),
    ("text-8xl", &["font-size", "line-height"]),
    ("text-9xl", &["font-size", "line-height"]),
    ("text-s", &["font-size", "line-height"]),

    // -- Font weight (font-bold, font-semibold, etc.) --
    ("font-thin", &["font-weight"]),
    ("font-extralight", &["font-weight"]),
    ("font-light", &["font-weight"]),
    ("font-normal", &["font-weight"]),
    ("font-medium", &["font-weight"]),
    ("font-semibold", &["font-weight"]),
    ("font-bold", &["font-weight"]),
    ("font-extrabold", &["font-weight"]),
    ("font-black", &["font-weight"]),
    ("font", &["font-weight"]),

    // -- Line height --
    ("leading", &["line-height"]),

    // -- Letter spacing --
    ("tracking", &["letter-spacing"]),

    // -- Text wrap --
    ("text-wrap", &["text-wrap"]),
    ("text-nowrap", &["text-wrap"]),
    ("text-balance", &["text-wrap"]),
    ("text-pretty", &["text-wrap"]),

    // -- Overflow wrap / word break --
    ("break-normal", &["overflow-wrap"]),
    ("break-all", &["word-break"]),
    ("break-keep", &["word-break"]),
    ("wrap-anywhere", &["overflow-wrap"]),
    ("wrap-break-word", &["overflow-wrap"]),
    ("wrap-normal", &["overflow-wrap"]),

    // -- Text overflow --
    ("text-ellipsis", &["text-overflow"]),
    ("text-clip", &["text-overflow"]),

    // -- Hyphens --
    ("hyphens-none", &["hyphens"]),
    ("hyphens-manual", &["hyphens"]),
    ("hyphens-auto", &["hyphens"]),

    // -- White space --
    ("whitespace-normal", &["white-space"]),
    ("whitespace-nowrap", &["white-space"]),
    ("whitespace-pre", &["white-space"]),
    ("whitespace-pre-line", &["white-space"]),
    ("whitespace-pre-wrap", &["white-space"]),
    ("whitespace-break-spaces", &["white-space"]),

    // -- Color --
    // Note: `text-{color}` is ambiguous with `text-{size}`. We resolve this
    // by matching specific color-like patterns in the lookup function.

    // -- Text transform --
    ("uppercase", &["text-transform"]),
    ("lowercase", &["text-transform"]),
    ("capitalize", &["text-transform"]),
    ("normal-case", &["text-transform"]),

    // -- Font style --
    ("italic", &["font-style"]),
    ("not-italic", &["font-style"]),

    // -- Font stretch --
    ("font-stretch", &["font-stretch"]),

    // -- Font variant numeric --
    ("normal-nums", &["font-variant-numeric"]),
    ("ordinal", &["font-variant-numeric"]),
    ("slashed-zero", &["font-variant-numeric"]),
    ("lining-nums", &["font-variant-numeric"]),
    ("oldstyle-nums", &["font-variant-numeric"]),
    ("proportional-nums", &["font-variant-numeric"]),
    ("tabular-nums", &["font-variant-numeric"]),
    ("diagonal-fractions", &["font-variant-numeric"]),
    ("stacked-fractions", &["font-variant-numeric"]),

    // -- Text decoration line --
    ("underline", &["text-decoration-line"]),
    ("overline", &["text-decoration-line"]),
    ("line-through", &["text-decoration-line"]),
    ("no-underline", &["text-decoration-line"]),

    // -- Text decoration color / style / thickness --
    ("decoration-solid", &["text-decoration-style"]),
    ("decoration-double", &["text-decoration-style"]),
    ("decoration-dotted", &["text-decoration-style"]),
    ("decoration-dashed", &["text-decoration-style"]),
    ("decoration-wavy", &["text-decoration-style"]),
    ("decoration-auto", &["text-decoration-thickness"]),
    ("decoration-from-font", &["text-decoration-thickness"]),
    ("decoration", &["text-decoration-thickness"]),

    // -- Text underline offset --
    ("underline-offset", &["text-underline-offset"]),

    // -- Font smoothing --
    ("antialiased", &["-webkit-font-smoothing"]),
    ("subpixel-antialiased", &["-webkit-font-smoothing"]),

    // -- Placeholder color (--tw-sort: placeholder-color) --
    ("placeholder", &["placeholder-color"]),

    // -- Caret color --
    ("caret", &["caret-color"]),

    // -- Accent color --
    ("accent", &["accent-color"]),

    // -- Color scheme --
    ("scheme-normal", &["color-scheme"]),
    ("scheme-dark", &["color-scheme"]),
    ("scheme-light", &["color-scheme"]),
    ("scheme-light-dark", &["color-scheme"]),
    ("scheme-only-dark", &["color-scheme"]),
    ("scheme-only-light", &["color-scheme"]),

    // -- Opacity --
    ("opacity", &["opacity"]),

    // -- Blend modes --
    ("bg-blend-normal", &["background-blend-mode"]),
    ("bg-blend-multiply", &["background-blend-mode"]),
    ("bg-blend-screen", &["background-blend-mode"]),
    ("bg-blend-overlay", &["background-blend-mode"]),
    ("bg-blend-darken", &["background-blend-mode"]),
    ("bg-blend-lighten", &["background-blend-mode"]),
    ("bg-blend-color-dodge", &["background-blend-mode"]),
    ("bg-blend-color-burn", &["background-blend-mode"]),
    ("bg-blend-hard-light", &["background-blend-mode"]),
    ("bg-blend-soft-light", &["background-blend-mode"]),
    ("bg-blend-difference", &["background-blend-mode"]),
    ("bg-blend-exclusion", &["background-blend-mode"]),
    ("bg-blend-hue", &["background-blend-mode"]),
    ("bg-blend-saturation", &["background-blend-mode"]),
    ("bg-blend-color", &["background-blend-mode"]),
    ("bg-blend-luminosity", &["background-blend-mode"]),
    ("mix-blend-normal", &["mix-blend-mode"]),
    ("mix-blend-multiply", &["mix-blend-mode"]),
    ("mix-blend-screen", &["mix-blend-mode"]),
    ("mix-blend-overlay", &["mix-blend-mode"]),
    ("mix-blend-darken", &["mix-blend-mode"]),
    ("mix-blend-lighten", &["mix-blend-mode"]),
    ("mix-blend-color-dodge", &["mix-blend-mode"]),
    ("mix-blend-color-burn", &["mix-blend-mode"]),
    ("mix-blend-hard-light", &["mix-blend-mode"]),
    ("mix-blend-soft-light", &["mix-blend-mode"]),
    ("mix-blend-difference", &["mix-blend-mode"]),
    ("mix-blend-exclusion", &["mix-blend-mode"]),
    ("mix-blend-hue", &["mix-blend-mode"]),
    ("mix-blend-saturation", &["mix-blend-mode"]),
    ("mix-blend-color", &["mix-blend-mode"]),
    ("mix-blend-luminosity", &["mix-blend-mode"]),

    // -- Box shadow --
    ("shadow", &["--tw-shadow", "box-shadow"]),
    ("shadow-initial", &["--tw-shadow-color"]),
    ("inset-shadow", &["--tw-inset-shadow", "box-shadow"]),
    ("inset-shadow-initial", &["--tw-inset-shadow-color"]),

    // -- Ring --
    // `ring` (bare) and `ring-<number>` = ring width (--tw-ring-shadow, box-shadow).
    // `ring-<color>` = ring color (--tw-ring-color).
    // Disambiguation between width and color is handled in `lookup_sort_key`.
    ("ring-inset", &["--tw-ring-shadow"]),
    ("ring", &["--tw-ring-shadow", "box-shadow"]),
    ("inset-ring", &["--tw-inset-ring-shadow", "box-shadow"]),
    ("ring-offset", &["--tw-ring-offset-width"]),

    // -- Outline --
    // Note: outline-style variants (hidden/none/solid/dashed/dotted/double)
    // generate `outline-style` which is NOT in PROPERTY_ORDER. In Tailwind's
    // sorting, this means they get an empty sort key (with count > 0), so they
    // sort after all classes with actual property indices. We map them to
    // "outline-style" which won't be found in PROPERTY_INDEX → empty order [].
    ("outline-hidden", &["outline-style"]),
    ("outline-none", &["outline-style"]),
    ("outline-solid", &["outline-style"]),
    ("outline-dashed", &["outline-style"]),
    ("outline-dotted", &["outline-style"]),
    ("outline-double", &["outline-style"]),
    ("outline", &["outline-width"]),
    ("outline-offset", &["outline-offset"]),

    // -- Filter --
    ("blur", &["--tw-blur", "filter"]),
    ("brightness", &["--tw-brightness", "filter"]),
    ("contrast", &["--tw-contrast", "filter"]),
    ("drop-shadow", &["--tw-drop-shadow", "filter"]),
    ("drop-shadow-none", &["--tw-drop-shadow", "filter"]),
    ("grayscale", &["--tw-grayscale", "filter"]),
    ("hue-rotate", &["--tw-hue-rotate", "filter"]),
    ("invert", &["--tw-invert", "filter"]),
    ("saturate", &["--tw-saturate", "filter"]),
    ("sepia", &["--tw-sepia", "filter"]),
    ("filter", &["filter"]),

    // -- Backdrop filter --
    ("backdrop-blur", &["--tw-backdrop-blur", "backdrop-filter"]),
    ("backdrop-brightness", &["--tw-backdrop-brightness", "backdrop-filter"]),
    ("backdrop-contrast", &["--tw-backdrop-contrast", "backdrop-filter"]),
    ("backdrop-grayscale", &["--tw-backdrop-grayscale", "backdrop-filter"]),
    ("backdrop-hue-rotate", &["--tw-backdrop-hue-rotate", "backdrop-filter"]),
    ("backdrop-invert", &["--tw-backdrop-invert", "backdrop-filter"]),
    ("backdrop-opacity", &["--tw-backdrop-opacity", "backdrop-filter"]),
    ("backdrop-saturate", &["--tw-backdrop-saturate", "backdrop-filter"]),
    ("backdrop-sepia", &["--tw-backdrop-sepia", "backdrop-filter"]),
    ("backdrop-filter", &["backdrop-filter"]),

    // -- Transition --
    // Bare `transition` and the static variants (colors, opacity, shadow, transform)
    // generate three declarations: transition-property, transition-timing-function,
    // and transition-duration. We list all three for correct sort ordering.
    ("transition", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-all", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-colors", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-opacity", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-shadow", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-transform", &["transition-property", "transition-timing-function", "transition-duration"]),
    ("transition-none", &["transition-property"]),
    ("transition-discrete", &["transition-behavior"]),
    ("transition-normal", &["transition-behavior"]),
    ("delay", &["transition-delay"]),
    ("duration", &["transition-duration"]),
    ("ease", &["transition-timing-function"]),

    // -- Will change --
    ("will-change", &["will-change"]),
    ("will-change-auto", &["will-change"]),
    ("will-change-scroll", &["will-change"]),
    ("will-change-contents", &["will-change"]),
    ("will-change-transform", &["will-change"]),

    // -- Contain --
    ("contain-none", &["contain"]),
    ("contain-content", &["contain"]),
    ("contain-strict", &["contain"]),
    ("contain-size", &["contain"]),
    ("contain-inline-size", &["contain"]),
    ("contain-layout", &["contain"]),
    ("contain-paint", &["contain"]),
    ("contain-style", &["contain"]),
    ("contain", &["contain"]),

    // -- Content --
    ("content-none", &["content"]),
    ("content", &["content"]),

    // -- User select (user-select is NOT in PROPERTY_ORDER, so these get empty
    // order arrays but count > 0, which makes them sort after all classes with
    // property indices — matching Tailwind's behavior) --
    ("select-none", &["user-select", "-webkit-user-select"]),
    ("select-text", &["user-select", "-webkit-user-select"]),
    ("select-all", &["user-select", "-webkit-user-select"]),
    ("select-auto", &["user-select", "-webkit-user-select"]),
    ("select-contain", &["user-select", "-webkit-user-select"]),

    // -- Forced color adjust --
    ("forced-color-adjust-none", &["forced-color-adjust"]),
    ("forced-color-adjust-auto", &["forced-color-adjust"]),

    // -- Accessibility (sr-only sorts by position — first recognized property) --
    ("sr-only", &["position"]),
    ("not-sr-only", &["position"]),
];

// ---------------------------------------------------------------------------
// Prefix lookup — built once at startup from CLASS_TO_PROPERTIES
// ---------------------------------------------------------------------------

/// For each class prefix, store the sorted+deduped property indices as the sort key.
/// This mirrors Tailwind's `getPropertySort` which returns `{ order: number[], count: number }`.
struct SortKey {
    /// Sorted, deduplicated property indices (ascending).
    order: Vec<usize>,
    /// Total number of CSS declarations (= number of properties listed, used for tie-breaking).
    count: usize,
}

static PREFIX_SORT_KEYS: Lazy<HashMap<&'static str, SortKey>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(CLASS_TO_PROPERTIES.len());
    for &(prefix, properties) in CLASS_TO_PROPERTIES {
        let mut indices: Vec<usize> = properties
            .iter()
            .filter_map(|prop| PROPERTY_INDEX.get(prop).copied())
            .collect();
        indices.sort_unstable();
        indices.dedup();
        let count = properties.len();
        map.insert(prefix, SortKey { order: indices, count });
    }
    map
});

// ---------------------------------------------------------------------------
// Sort key computation for a single class
// ---------------------------------------------------------------------------

/// The sort key for a class: (variant_count, property_order, property_count, base_class).
/// - `variant_count`: number of variant prefixes (e.g., `hover:md:` = 2). More variants = later.
/// - `property_order`: sorted property indices (ascending), or `None` for unknown classes.
/// - `property_count`: total number of properties (more = earlier, matching Tailwind's tie-break).
/// - `base_class`: the class name without variants, for alphabetical tie-breaking.
#[derive(Debug)]
struct ClassSortKey<'a> {
    variant_count: usize,
    order: Option<&'a [usize]>,
    count: usize,
    base_class: &'a str,
    original_index: usize,
}

/// Strip variant prefixes from a class name to find the base utility.
///
/// Tailwind variant prefixes are colon-separated: `hover:`, `md:`, `dark:`, etc.
/// We split on the LAST colon to get the base utility name.
///
/// Special handling for arbitrary variants like `[&>svg]:` — we skip over
/// brackets to avoid splitting inside them.
fn strip_variants(class: &str) -> (usize, &str) {
    // Count colons that are variant separators (not inside brackets)
    let mut variant_count = 0;
    let mut last_colon_end = 0;
    let mut bracket_depth: u32 = 0;

    for (i, ch) in class.char_indices() {
        match ch {
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ':' if bracket_depth == 0 => {
                variant_count += 1;
                last_colon_end = i + 1;
            }
            _ => {}
        }
    }

    if variant_count > 0 {
        (variant_count, &class[last_colon_end..])
    } else {
        (0, class)
    }
}

// ---------------------------------------------------------------------------
// Disambiguation: Ambiguous prefix → color override lookup
//
// Some Tailwind prefixes (like "text", "border", "ring", "shadow", "divide")
// generate different CSS properties depending on the value (size vs color).
// The PREFIX_SORT_KEYS table stores the "default" mapping. When prefix
// stripping lands on one of these ambiguous prefixes, we check the
// value suffix to determine if it's a color or the default.
//
// Approach: if the full class matches the default mapping's expected pattern
// (e.g., numeric size, known keyword), keep it. Otherwise, assume it's a
// color value and use the color sort key.
// ---------------------------------------------------------------------------

/// Alternate sort keys for ambiguous prefixes when the value is a color.
static DISAMBIG_COLOR_KEYS: Lazy<HashMap<&'static str, SortKey>> = Lazy::new(|| {
    let mut map = HashMap::new();

    let make_key = |properties: &[&str]| -> SortKey {
        let mut indices: Vec<usize> = properties
            .iter()
            .filter_map(|prop| PROPERTY_INDEX.get(prop).copied())
            .collect();
        indices.sort_unstable();
        indices.dedup();
        SortKey { order: indices, count: properties.len() }
    };

    // border-<color> → border-color (instead of border-width)
    map.insert("border", make_key(&["border-color"]));
    // ring-<color> → --tw-ring-color (instead of --tw-ring-shadow, box-shadow)
    map.insert("ring", make_key(&["--tw-ring-color"]));
    // inset-ring-<color>
    map.insert("inset-ring", make_key(&["--tw-inset-ring-color"]));
    // shadow-<color> → --tw-shadow-color (instead of --tw-shadow, box-shadow)
    map.insert("shadow", make_key(&["--tw-shadow-color"]));
    // inset-shadow-<color>
    map.insert("inset-shadow", make_key(&["--tw-inset-shadow-color"]));
    // divide-<color> → divide-color (instead of divide-x-width)
    map.insert("divide", make_key(&["divide-color"]));

    map
});

/// Check if a suffix after an ambiguous prefix looks like a numeric/keyword value
/// (not a color). Returns true if the value is a width/size/keyword.
fn is_numeric_or_keyword_value(prefix: &str, suffix: &str) -> bool {
    if suffix.is_empty() {
        return true; // bare prefix (e.g., "ring", "border") = default mapping
    }

    // Arbitrary values in brackets: check if they contain numbers/units
    if suffix.starts_with('[') {
        // Arbitrary numeric: [3px], [0.5rem], [2], [50%], [calc(...)]
        let inner = suffix.trim_start_matches('[').trim_end_matches(']');
        return inner.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c == '-')
            || inner.starts_with("calc(")
            || inner.starts_with("var(")
            || inner.ends_with("px")
            || inner.ends_with("rem")
            || inner.ends_with("em")
            || inner.ends_with('%');
    }

    // Check prefix-specific patterns
    match prefix {
        "border" => {
            // border-0 through border-8 are explicit entries; this handles
            // arbitrary widths and ensures border-<color> gets color sort key.
            // The value is a width if it's numeric.
            suffix.parse::<f64>().is_ok()
        }
        "ring" | "inset-ring" => {
            // ring-0, ring-1, ring-2, ring-4, ring-8, ring-inset
            suffix.parse::<f64>().is_ok() || suffix == "inset"
        }
        "shadow" | "inset-shadow" => {
            // shadow-sm, shadow-md, shadow-lg, shadow-xl, shadow-2xl, shadow-none, shadow-inner
            matches!(suffix, "sm" | "md" | "lg" | "xl" | "2xl" | "none" | "inner" | "initial")
        }
        "divide" => {
            // divide-x, divide-y, divide-solid, divide-dashed, etc.
            // These have explicit entries in CLASS_TO_PROPERTIES, so they won't reach
            // disambiguation. But just in case:
            matches!(suffix, "x" | "y" | "solid" | "dashed" | "dotted" | "double" | "none")
        }
        _ => false,
    }
}

/// Look up the sort key for a base utility class (without variants).
///
/// Strategy:
/// 1. Try exact match in PREFIX_SORT_KEYS.
/// 2. Try progressively shorter prefixes by removing from the end at `-` boundaries.
///    e.g., `p-4` → try `p-4`, then `p`.
/// 3. Handle negative values: `-m-4` → strip leading `-`, try `m-4`, then `m`.
/// 4. Disambiguate ambiguous prefixes (text, border, ring, shadow, divide)
///    by checking if the value suffix indicates a color.
fn lookup_sort_key(base_class: &str) -> Option<&'static SortKey> {
    // 1. Exact match
    if let Some(key) = PREFIX_SORT_KEYS.get(base_class) {
        return Some(key);
    }

    // 2. Handle negative prefix: -m-4 → m-4 → m
    let effective = if base_class.starts_with('-') {
        &base_class[1..]
    } else {
        base_class
    };

    if effective != base_class {
        if let Some(key) = PREFIX_SORT_KEYS.get(effective) {
            return Some(key);
        }
    }

    // 3. Progressive prefix stripping at `-` boundaries
    let mut candidate = effective;
    while let Some(dash_pos) = candidate.rfind('-') {
        candidate = &candidate[..dash_pos];
        if let Some(key) = PREFIX_SORT_KEYS.get(candidate) {
            // 4. Disambiguate: check if this is an ambiguous prefix with a color value
            if let Some(color_key) = DISAMBIG_COLOR_KEYS.get(candidate) {
                let suffix = &effective[candidate.len() + 1..]; // part after "prefix-"
                if !is_numeric_or_keyword_value(candidate, suffix) {
                    return Some(color_key);
                }
            }
            return Some(key);
        }
    }

    None
}

/// Compare two class sort keys using Tailwind's algorithm from compile.ts:
/// 1. Sort by variant count (fewer variants first)
/// 2. Sort by property indices (element-by-element comparison)
/// 3. More properties → earlier (tie-break)
/// 4. Alphabetical by class name (final tie-break)
/// 5. Unknown classes sort last, preserving original order
fn compare_classes(a: &ClassSortKey, b: &ClassSortKey) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // Unknown classes sort BEFORE known classes (matching prettier-plugin-tailwindcss
    // behavior where getClassOrder returns null → sorts first, preserving relative order).
    match (a.order, b.order) {
        (None, None) => return a.original_index.cmp(&b.original_index),
        (None, Some(_)) => return Ordering::Less,
        (Some(_), None) => return Ordering::Greater,
        (Some(a_order), Some(b_order)) => {
            // Sort by variant count first
            let variant_cmp = a.variant_count.cmp(&b.variant_count);
            if variant_cmp != Ordering::Equal {
                return variant_cmp;
            }

            // Compare property indices element-by-element
            let mut offset = 0;
            while offset < a_order.len() && offset < b_order.len() {
                if a_order[offset] != b_order[offset] {
                    return a_order[offset].cmp(&b_order[offset]);
                }
                offset += 1;
            }

            // If all compared elements are equal, the one with MORE properties comes first
            // (matching Tailwind's `zSorting.properties.count - aSorting.properties.count`)
            let count_cmp = b.count.cmp(&a.count);
            if count_cmp != Ordering::Equal {
                return count_cmp;
            }

            // If same property indices and count, sort by property vector length
            // (more indices = more specific, comes first)
            let len_cmp = b_order.len().cmp(&a_order.len());
            if len_cmp != Ordering::Equal {
                return len_cmp;
            }

            // Final tie-break: alphabetical by base class name
            a.base_class.cmp(b.base_class)
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SortOptions {
    pub preserve_duplicates: bool,
    pub preserve_whitespace: bool,
}

/// A Tailwind CSS class sorter using CSS property-based ordering.
///
/// Implements the same sorting algorithm as Tailwind CSS v4's compiler.
/// See `tailwindcss/packages/tailwindcss/src/compile.ts` lines 83-115 and 325-367.
#[derive(Debug, Clone)]
pub struct ClassSorter {
    pub options: SortOptions,
}

impl ClassSorter {
    pub fn new(options: SortOptions) -> Self {
        Self { options }
    }

    /// Sort a list of class-attribute strings (space-separated class names each).
    /// This is the `TailwindCallback` interface: takes `Vec<String>`, returns `Vec<String>`.
    pub fn sort_class_attributes(&self, class_attrs: Vec<String>) -> Vec<String> {
        class_attrs
            .into_iter()
            .map(|attr| self.sort_class_string(attr))
            .collect()
    }

    /// Sort a single space-separated class string.
    ///
    /// Matches the behavior of `prettier-plugin-tailwindcss`'s `sortClasses`:
    /// known Tailwind utilities are sorted by CSS property order, unknown classes
    /// are pushed to the end preserving their relative order.
    pub fn sort_class_string(&self, class_str: String) -> String {
        if class_str.is_empty() {
            return class_str;
        }

        // Whitespace-only → normalize to single space (matching prettier-plugin-tailwindcss)
        let trimmed = class_str.trim();
        if trimmed.is_empty() {
            return " ".to_string();
        }

        // Preserve leading/trailing whitespace if option set
        let has_leading_ws = class_str.starts_with(|c: char| c.is_ascii_whitespace());
        let has_trailing_ws = class_str.ends_with(|c: char| c.is_ascii_whitespace());

        let mut classes: Vec<&str> = trimmed.split_ascii_whitespace().collect();

        if !self.options.preserve_duplicates {
            classes = remove_duplicates(classes);
        }

        // Build sort keys
        let mut sort_entries: Vec<ClassSortKey> = classes
            .iter()
            .enumerate()
            .map(|(i, &cls)| {
                let (variant_count, base) = strip_variants(cls);
                let sort_key = lookup_sort_key(base);
                ClassSortKey {
                    variant_count,
                    order: sort_key.map(|k| k.order.as_slice()),
                    count: sort_key.map_or(0, |k| k.count),
                    base_class: base,
                    original_index: i,
                }
            })
            .collect();

        // Stable sort using Tailwind's comparison algorithm
        sort_entries.sort_by(compare_classes);

        // Rebuild the class string in sorted order
        let sorted_classes: Vec<&str> = sort_entries
            .iter()
            .map(|entry| classes[entry.original_index])
            .collect();

        let mut result = sorted_classes.join(" ");

        if self.options.preserve_whitespace {
            if has_leading_ws {
                result.insert(0, ' ');
            }
            if has_trailing_ws {
                result.push(' ');
            }
        }

        result
    }
}

fn remove_duplicates<'a>(classes: Vec<&'a str>) -> Vec<&'a str> {
    let mut seen = std::collections::HashSet::new();
    classes.into_iter().filter(|cls| seen.insert(*cls)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorter() -> ClassSorter {
        ClassSorter::new(SortOptions::default())
    }

    // --- Core sorting ---

    #[test]
    fn sorts_flex_before_padding() {
        // flex → display (idx ~35), p-4 → padding (idx ~252)
        let s = sorter();
        assert_eq!(s.sort_class_string("p-4 flex".to_string()), "flex p-4");
    }

    #[test]
    fn sorts_display_before_flex_property() {
        // flex (display) before flex-1 prefix strips to flex → display
        // but flex-auto is a different entry mapping to CSS flex property
        let s = sorter();
        let result = s.sort_class_string("flex-auto flex".to_string());
        assert_eq!(result, "flex flex-auto");
    }

    #[test]
    fn already_sorted_unchanged() {
        let s = sorter();
        assert_eq!(s.sort_class_string("flex p-4".to_string()), "flex p-4");
    }

    #[test]
    fn removes_duplicates_by_default() {
        let s = sorter();
        assert_eq!(s.sort_class_string("flex p-4 flex".to_string()), "flex p-4");
    }

    #[test]
    fn preserves_duplicates_when_option_set() {
        let s = ClassSorter::new(SortOptions { preserve_duplicates: true, ..Default::default() });
        assert_eq!(s.sort_class_string("flex p-4 flex p-4".to_string()), "flex flex p-4 p-4");
    }

    #[test]
    fn unknown_classes_sort_first() {
        // Unknown classes sort before known classes (matching prettier-plugin-tailwindcss)
        let s = sorter();
        let result = s.sort_class_string("custom-class flex another-custom p-4".to_string());
        assert!(result.ends_with("flex p-4"), "got: {result}");
        assert!(result.starts_with("custom-class another-custom"), "got: {result}");
    }

    #[test]
    fn preserve_relative_order_of_unknowns() {
        let s = sorter();
        let result = s.sort_class_string("zzz-unknown flex aaa-unknown".to_string());
        assert_eq!(result, "zzz-unknown aaa-unknown flex");
    }

    #[test]
    fn preserves_whitespace_when_option_set() {
        let s = ClassSorter::new(SortOptions { preserve_whitespace: true, ..Default::default() });
        assert_eq!(s.sort_class_string("  p-4 flex  ".to_string()), " flex p-4 ");
    }

    #[test]
    fn whitespace_only_normalizes_to_single_space() {
        let s = sorter();
        assert_eq!(s.sort_class_string("   ".to_string()), " ");
    }

    #[test]
    fn empty_string_unchanged() {
        let s = sorter();
        assert_eq!(s.sort_class_string("".to_string()), "");
    }

    #[test]
    fn single_class_unchanged() {
        let s = sorter();
        assert_eq!(s.sort_class_string("flex".to_string()), "flex");
    }

    // --- Property order tests (verifying category ordering) ---

    #[test]
    fn position_before_margin() {
        // position (idx 3) before margin (idx ~25)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("m-4 absolute".to_string()),
            "absolute m-4"
        );
    }

    #[test]
    fn margin_before_display() {
        // margin (idx ~25) before display (idx ~35)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("flex m-4".to_string()),
            "m-4 flex"
        );
    }

    #[test]
    fn display_before_width() {
        // display (idx ~35) before width (idx ~41)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("w-full flex".to_string()),
            "flex w-full"
        );
    }

    #[test]
    fn width_before_padding() {
        // width (idx ~41) before padding (idx ~252)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("p-4 w-10".to_string()),
            "w-10 p-4"
        );
    }

    #[test]
    fn padding_before_text_color() {
        // padding (idx ~252) before font-size (idx ~262)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("text-sm p-4".to_string()),
            "p-4 text-sm"
        );
    }

    #[test]
    fn background_color_before_padding() {
        // background-color (idx ~181) before padding (idx ~252)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("p-4 bg-red-500".to_string()),
            "bg-red-500 p-4"
        );
    }

    // --- Directional utilities ---

    #[test]
    fn margin_shorthand_before_directional() {
        // margin (idx ~25) before margin-inline (idx ~26) before margin-top (idx ~30)
        let s = sorter();
        assert_eq!(
            s.sort_class_string("mt-2 mx-4 m-0".to_string()),
            "m-0 mx-4 mt-2"
        );
    }

    #[test]
    fn padding_shorthand_before_directional() {
        let s = sorter();
        assert_eq!(
            s.sort_class_string("pt-2 px-4 p-0".to_string()),
            "p-0 px-4 pt-2"
        );
    }

    // --- Negative values ---

    #[test]
    fn negative_margin() {
        let s = sorter();
        assert_eq!(
            s.sort_class_string("p-4 -m-2".to_string()),
            "-m-2 p-4"
        );
    }

    // --- Variant handling ---

    #[test]
    fn variants_sort_after_base() {
        let s = sorter();
        let result = s.sort_class_string("hover:bg-blue-500 flex p-4".to_string());
        let parts: Vec<&str> = result.split_whitespace().collect();
        let flex_pos = parts.iter().position(|&c| c == "flex").unwrap();
        let hover_pos = parts.iter().position(|&c| c == "hover:bg-blue-500").unwrap();
        assert!(flex_pos < hover_pos, "flex should come before hover:bg-blue-500, got: {result}");
    }

    #[test]
    fn multiple_variants_sort_after_single_variant() {
        let s = sorter();
        let result = s.sort_class_string("dark:hover:bg-red-500 hover:bg-blue-500 flex".to_string());
        let parts: Vec<&str> = result.split_whitespace().collect();
        let flex_pos = parts.iter().position(|&c| c == "flex").unwrap();
        let hover_pos = parts.iter().position(|&c| c == "hover:bg-blue-500").unwrap();
        let dark_hover_pos = parts.iter().position(|&c| c == "dark:hover:bg-red-500").unwrap();
        assert!(flex_pos < hover_pos, "got: {result}");
        assert!(hover_pos < dark_hover_pos, "got: {result}");
    }

    // --- Comprehensive example ---

    #[test]
    fn comprehensive_sort() {
        let s = sorter();
        let input = "text-white text-sm font-medium hover:bg-blue-500 disabled:opacity-50 gap-2 items-center flex py-2 px-4 bg-blue-500 border rounded-lg";
        let result = s.sort_class_string(input.to_string());
        let parts: Vec<&str> = result.split_whitespace().collect();

        // Verify key ordering relationships
        let pos = |name: &str| parts.iter().position(|&c| c == name).unwrap();

        // display before layout properties
        assert!(pos("flex") < pos("items-center"), "flex < items-center: {result}");
        assert!(pos("items-center") < pos("gap-2"), "items-center < gap-2: {result}");
        // border before background
        assert!(pos("rounded-lg") < pos("border"), "rounded-lg < border: {result}");
        assert!(pos("border") < pos("bg-blue-500"), "border < bg-blue-500: {result}");
        // padding after background
        assert!(pos("bg-blue-500") < pos("px-4"), "bg-blue-500 < px-4: {result}");
        assert!(pos("px-4") < pos("py-2"), "px-4 < py-2: {result}");
        // text after padding
        assert!(pos("py-2") < pos("text-sm"), "py-2 < text-sm: {result}");
        // variants last
        assert!(pos("text-white") < pos("hover:bg-blue-500"), "base < hover variant: {result}");
        assert!(pos("text-white") < pos("disabled:opacity-50"), "base < disabled variant: {result}");
    }

    // --- Batch sort ---

    #[test]
    fn batch_sort() {
        let s = sorter();
        let result = s.sort_class_attributes(vec![
            "p-4 flex".to_string(),
            "text-white bg-red-500".to_string(),
        ]);
        assert_eq!(result[0], "flex p-4");
        assert_eq!(result[1], "bg-red-500 text-white");
    }

    // --- strip_variants ---

    #[test]
    fn strip_variants_none() {
        assert_eq!(strip_variants("flex"), (0, "flex"));
    }

    #[test]
    fn strip_variants_single() {
        assert_eq!(strip_variants("hover:flex"), (1, "flex"));
    }

    #[test]
    fn strip_variants_multiple() {
        assert_eq!(strip_variants("dark:hover:flex"), (2, "flex"));
    }

    #[test]
    fn strip_variants_arbitrary() {
        // Brackets should not count as variant separators
        assert_eq!(strip_variants("[&>svg]:flex"), (1, "flex"));
    }

    // --- lookup_sort_key ---

    #[test]
    fn lookup_exact_match() {
        assert!(lookup_sort_key("flex").is_some());
        assert!(lookup_sort_key("hidden").is_some());
    }

    #[test]
    fn lookup_prefix_strip() {
        // p-4 → strips to "p" which maps to padding
        assert!(lookup_sort_key("p-4").is_some());
        assert!(lookup_sort_key("m-auto").is_some());
        assert!(lookup_sort_key("bg-red-500").is_some());
    }

    #[test]
    fn lookup_negative() {
        // -m-4 → strips "-" → "m-4" → "m"
        assert!(lookup_sort_key("-m-4").is_some());
    }

    #[test]
    fn lookup_unknown() {
        // "xyz-whatever" doesn't match any Tailwind prefix
        assert!(lookup_sort_key("xyz-whatever").is_none());
        // Note: "my-custom-class" would match "my" (margin-block) via prefix
        // stripping — this is expected since we can't distinguish custom classes
        // from Tailwind utilities without compiling CSS.
    }
}
