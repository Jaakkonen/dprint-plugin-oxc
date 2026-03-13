// Copyright 2025 Anomaly Security Inc.
// Author: Anomaly Labs
//
// Licensed under the MIT License (the "License");
// you may not use this file except in compliance with the License.

//! Integration tests for the property-based Tailwind CSS v4 class sorter.
//!
//! Each test references the source-of-truth files in the Tailwind CSS v4 repo:
//! - `property-order.ts` — canonical CSS property ordering (~416 entries)
//! - `utilities.ts` — class prefix → CSS property mapping
//! - `compile.ts` — sorting algorithm (getPropertySort, lines 325-367)

use tailwind_sorter::{ClassSorter, SortOptions};

fn sorter() -> ClassSorter {
    ClassSorter::new(SortOptions::default())
}

fn sort(input: &str) -> String {
    sorter().sort_class_string(input.to_string())
}

// ---------------------------------------------------------------------------
// Basic functionality
// ---------------------------------------------------------------------------

#[test]
fn empty_string() {
    assert_eq!(sort(""), "");
}

#[test]
fn single_class() {
    assert_eq!(sort("flex"), "flex");
}

#[test]
fn already_sorted() {
    assert_eq!(sort("flex p-4"), "flex p-4");
}

#[test]
fn whitespace_only() {
    assert_eq!(sort("   "), " ");
}

// ---------------------------------------------------------------------------
// Category ordering — verifying the CSS property order from property-order.ts
//
// The sort order follows the property positions:
//   position(3) < inset(4) < z-index(14) < margin(25) < display(35)
//   < height(38) < width(41) < flex(44) < border-radius(137)
//   < border-width(153) < background-color(181) < padding(252)
//   < font-size(262) < color(278) < opacity(297) < box-shadow(302)
//   < filter(328) < transition-property(340)
// ---------------------------------------------------------------------------

#[test]
fn position_before_inset() {
    // position(3) < inset(4) in property-order.ts
    assert_eq!(sort("inset-0 absolute"), "absolute inset-0");
}

#[test]
fn inset_before_z_index() {
    // inset(4) < z-index(14)
    assert_eq!(sort("z-10 top-0"), "top-0 z-10");
}

#[test]
fn z_index_before_margin() {
    // z-index(14) < margin(25)
    assert_eq!(sort("m-4 z-50"), "z-50 m-4");
}

#[test]
fn margin_before_display() {
    // margin(25) < display(35)
    assert_eq!(sort("flex m-4"), "m-4 flex");
}

#[test]
fn display_before_height() {
    // display(35) < height(38)
    assert_eq!(sort("h-10 grid"), "grid h-10");
}

#[test]
fn height_before_width() {
    // height(38) < width(41) in property-order.ts
    assert_eq!(sort("w-full h-full"), "h-full w-full");
}

#[test]
fn display_before_flex_property() {
    // display(35) for "flex" vs flex CSS property(44) for "flex-auto"
    assert_eq!(sort("flex-auto flex"), "flex flex-auto");
}

#[test]
fn flex_property_before_border_radius() {
    // flex(44) < border-radius(137)
    assert_eq!(sort("rounded-lg flex-1"), "flex-1 rounded-lg");
}

#[test]
fn border_radius_before_border_width() {
    // border-radius(137) < border-width(153)
    assert_eq!(sort("border rounded-lg"), "rounded-lg border");
}

#[test]
fn border_width_before_background_color() {
    // border-width(153) < background-color(181)
    assert_eq!(sort("bg-blue-500 border-2"), "border-2 bg-blue-500");
}

#[test]
fn background_before_padding() {
    // background-color(181) < padding(252)
    assert_eq!(sort("p-4 bg-red-500"), "bg-red-500 p-4");
}

#[test]
fn padding_before_font_size() {
    // padding(252) < font-size(262)
    assert_eq!(sort("text-sm p-4"), "p-4 text-sm");
}

#[test]
fn font_size_before_opacity() {
    // font-size(262) < opacity(297)
    assert_eq!(sort("opacity-50 text-lg"), "text-lg opacity-50");
}

#[test]
fn opacity_before_shadow() {
    // opacity(297) < box-shadow(302) via --tw-shadow
    assert_eq!(sort("shadow-lg opacity-75"), "opacity-75 shadow-lg");
}

#[test]
fn shadow_before_filter() {
    // --tw-shadow(303) < --tw-blur(319) < filter(328)
    assert_eq!(sort("blur-sm shadow-md"), "shadow-md blur-sm");
}

#[test]
fn filter_before_transition() {
    // filter(328) < transition-property(340)
    assert_eq!(sort("transition blur-md"), "blur-md transition");
}

// ---------------------------------------------------------------------------
// Directional utilities — shorthand sorts before directional
// ---------------------------------------------------------------------------

#[test]
fn margin_shorthand_order() {
    // margin(25) < margin-inline(26) < margin-block(27) < margin-top(30)
    assert_eq!(sort("mt-2 mx-4 m-0 my-1"), "m-0 mx-4 my-1 mt-2");
}

#[test]
fn padding_shorthand_order() {
    // padding(252) < padding-inline(253) < padding-block(254) < padding-top(257)
    assert_eq!(sort("pt-2 px-4 p-0 py-1"), "p-0 px-4 py-1 pt-2");
}

#[test]
fn inset_directional_order() {
    // inset(4) < inset-inline(5) < inset-block(6) < top(9) < right(10) < bottom(11) < left(12)
    assert_eq!(
        sort("left-0 top-0 inset-x-0 inset-0 bottom-0 right-0"),
        "inset-0 inset-x-0 top-0 right-0 bottom-0 left-0"
    );
}

#[test]
fn border_width_directional() {
    // border-width(153) < border-inline-width(154) < border-top-width(158)
    assert_eq!(sort("border-t-2 border-x-2 border-2"), "border-2 border-x-2 border-t-2");
}

// ---------------------------------------------------------------------------
// Negative values
// ---------------------------------------------------------------------------

#[test]
fn negative_margin_sorts_correctly() {
    // -m-2 → strips "-" → "m-2" → "m" → margin
    assert_eq!(sort("p-4 -m-2"), "-m-2 p-4");
}

#[test]
fn negative_inset_sorts_correctly() {
    assert_eq!(sort("flex -top-1"), "-top-1 flex");
}

// ---------------------------------------------------------------------------
// Variant handling
// ---------------------------------------------------------------------------

#[test]
fn single_variant_sorts_after_base() {
    let result = sort("hover:bg-blue-500 flex p-4");
    assert!(result.starts_with("flex"), "got: {result}");
    assert!(result.ends_with("hover:bg-blue-500"), "got: {result}");
}

#[test]
fn multiple_variants_sort_by_count() {
    // 0 variants < 1 variant < 2 variants
    let result = sort("dark:hover:text-white hover:bg-blue-500 flex");
    let parts: Vec<&str> = result.split_whitespace().collect();
    assert_eq!(parts[0], "flex");  // 0 variants
    assert_eq!(parts[1], "hover:bg-blue-500");  // 1 variant
    assert_eq!(parts[2], "dark:hover:text-white");  // 2 variants
}

#[test]
fn variant_with_same_base_utility() {
    // base flex before hover:flex
    assert_eq!(sort("hover:flex flex"), "flex hover:flex");
}

#[test]
fn responsive_variants() {
    // md: and lg: are single variants, should sort after base
    let result = sort("lg:flex md:flex flex");
    let parts: Vec<&str> = result.split_whitespace().collect();
    assert_eq!(parts[0], "flex");
    // md and lg both have 1 variant, sorted alphabetically by base
    // both map to "flex" (display), so tie-break alphabetically on full class
}

// ---------------------------------------------------------------------------
// Unknown classes
// ---------------------------------------------------------------------------

#[test]
fn unknown_classes_sort_last_preserving_order() {
    assert_eq!(
        sort("zzz-unknown flex aaa-unknown p-4"),
        "flex p-4 zzz-unknown aaa-unknown"
    );
}

#[test]
fn all_unknown_classes_preserve_order() {
    assert_eq!(
        sort("charlie alpha bravo"),
        "charlie alpha bravo"
    );
}

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

#[test]
fn removes_duplicates_by_default() {
    assert_eq!(sort("flex p-4 flex"), "flex p-4");
}

#[test]
fn preserves_duplicates_when_option_set() {
    let s = ClassSorter::new(SortOptions {
        preserve_duplicates: true,
        ..Default::default()
    });
    assert_eq!(
        s.sort_class_string("flex p-4 flex p-4".to_string()),
        "flex flex p-4 p-4"
    );
}

// ---------------------------------------------------------------------------
// Whitespace handling
// ---------------------------------------------------------------------------

#[test]
fn preserves_whitespace_when_option_set() {
    let s = ClassSorter::new(SortOptions {
        preserve_whitespace: true,
        ..Default::default()
    });
    assert_eq!(
        s.sort_class_string("  p-4 flex  ".to_string()),
        " flex p-4 "
    );
}

// ---------------------------------------------------------------------------
// Batch sorting (the TailwindCallback interface)
// ---------------------------------------------------------------------------

#[test]
fn batch_sort_multiple_strings() {
    let s = sorter();
    let result = s.sort_class_attributes(vec![
        "p-4 flex".to_string(),
        "text-white bg-red-500".to_string(),
        "hover:opacity-80 shadow-lg rounded".to_string(),
    ]);
    assert_eq!(result[0], "flex p-4");
    assert_eq!(result[1], "bg-red-500 text-white");
    // rounded → border-radius, shadow-lg → box-shadow, hover:opacity-80 → variant
    assert!(result[2].starts_with("rounded"), "got: {}", result[2]);
}

// ---------------------------------------------------------------------------
// Comprehensive real-world examples
// ---------------------------------------------------------------------------

#[test]
fn button_component_classes() {
    // A typical button component from a design system
    let input = "text-white text-sm font-medium hover:bg-primary/80 disabled:opacity-50 gap-2 items-center inline-flex py-2 px-4 bg-primary border border-transparent rounded-lg transition-colors";
    let result = sort(input);
    let parts: Vec<&str> = result.split_whitespace().collect();

    let pos = |name: &str| {
        parts.iter().position(|&c| c == name)
            .unwrap_or_else(|| panic!("class '{name}' not found in: {result}"))
    };

    // Layout first
    assert!(pos("inline-flex") < pos("items-center"), "{result}");
    assert!(pos("items-center") < pos("gap-2"), "{result}");
    // Border before background
    assert!(pos("rounded-lg") < pos("border"), "{result}");
    assert!(pos("border") < pos("border-transparent"), "{result}");
    assert!(pos("border-transparent") < pos("bg-primary"), "{result}");
    // Padding after background
    assert!(pos("bg-primary") < pos("px-4"), "{result}");
    // Text after padding
    assert!(pos("py-2") < pos("text-sm"), "{result}");
    // Transition near end
    assert!(pos("font-medium") < pos("transition-colors"), "{result}");
    // Variants last
    assert!(pos("transition-colors") < pos("hover:bg-primary/80"), "{result}");
    assert!(pos("transition-colors") < pos("disabled:opacity-50"), "{result}");
}

#[test]
fn card_component_classes() {
    let input = "shadow-md p-6 rounded-xl bg-white border border-gray-200 w-full max-w-sm";
    let result = sort(input);
    let parts: Vec<&str> = result.split_whitespace().collect();

    let pos = |name: &str| {
        parts.iter().position(|&c| c == name)
            .unwrap_or_else(|| panic!("class '{name}' not found in: {result}"))
    };

    // Size before border
    assert!(pos("w-full") < pos("rounded-xl"), "{result}");
    assert!(pos("max-w-sm") < pos("rounded-xl"), "{result}");
    // Border radius before border width
    assert!(pos("rounded-xl") < pos("border"), "{result}");
    // Border before background
    assert!(pos("border") < pos("bg-white"), "{result}");
    // Background before padding
    assert!(pos("bg-white") < pos("p-6"), "{result}");
    // Shadow after text/opacity area
    assert!(pos("p-6") < pos("shadow-md"), "{result}");
}
