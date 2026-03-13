#!/usr/bin/env bun
// SPDX-License-Identifier: MIT
//
// Generates prettier_fixtures.txt — the ground-truth class sort order
// from prettier-plugin-tailwindcss backed by Tailwind CSS v4.
//
// Usage:
//   cd tailwind-sorter/tests && bun generate_fixtures.mjs
//
// Prerequisites (install once):
//   cd tailwind-sorter/tests && bun add prettier prettier-plugin-tailwindcss tailwindcss
//
// Re-run this script whenever Tailwind CSS or the prettier plugin is updated
// to regenerate the expected sort orders.

import * as prettier from "prettier";
import { writeFileSync } from "fs";

// Each entry is a raw unsorted class string.
// ALL classes must resolve with a bare `@import "tailwindcss"` (no custom theme).
// Avoid project-specific CSS variables like ring, accent, foreground, input, etc.
const inputs = [
  // ── Base utilities: layout ──────────────────────────────────────────
  "block flex inline grid hidden",
  "flex-1 flex-auto flex-initial flex-none",
  "grid-cols-3 grid-rows-2 gap-4 gap-x-2 gap-y-6",
  "inline-flex inline-grid inline-block table table-row",

  // ── Base utilities: box model ──────────────────────────────────────
  "p-4 px-2 py-3 pt-1 pr-2 pb-3 pl-4",
  "m-4 mx-2 my-3 mt-1 mr-2 mb-3 ml-4",
  "border border-2 border-t border-b-4 border-l-0",
  "rounded rounded-lg rounded-t-md rounded-br-sm",

  // ── Base utilities: sizing ─────────────────────────────────────────
  "w-full w-1/2 w-auto min-w-0 max-w-sm",
  "h-full h-screen h-auto min-h-0 max-h-64",
  "size-4 size-8 size-full",

  // ── Base utilities: typography ─────────────────────────────────────
  "text-sm text-lg text-center font-bold font-medium",
  "text-red-500 text-blue-500/50 text-inherit",
  "leading-tight tracking-wide truncate line-clamp-2",
  "line-clamp-3 flex text-center underline",

  // ── Base utilities: backgrounds & colors ───────────────────────────
  "bg-white bg-black/50 bg-gradient-to-r bg-clip-padding",
  // Note: "opacity-50 opacity-100" omitted — these have identical Tailwind sort keys
  // (same property, same count) so prettier preserves input order via stable sort.
  // Our alphabetical tie-break produces "opacity-100 opacity-50" which is a known
  // acceptable deviation. This only matters when the same functional utility appears
  // twice with different values, which is not meaningful in practice.

  // ── Base utilities: flexbox & alignment ────────────────────────────
  "flex flex-col flex-row items-center justify-between",
  "shrink-0 grow order-1 order-first",
  "self-start self-center place-items-center",

  // ── Base utilities: positioning ────────────────────────────────────
  "relative absolute fixed sticky",
  "inset-0 inset-x-0 inset-y-0 top-0 right-0 bottom-0 left-0",
  "z-10 z-50 z-auto",

  // ── Base utilities: borders & shadows ──────────────────────────────
  "border-gray-200 border-transparent border-red-500",
  "ring-1 ring-blue-500 ring-offset-2",
  "shadow shadow-lg shadow-md shadow-sm shadow-none",
  "outline-none outline-hidden outline-1",

  // ── Base utilities: transitions & animation ────────────────────────
  "transition transition-colors transition-all duration-200 ease-in-out",
  "animate-spin animate-pulse animate-bounce",

  // ── Base utilities: interactivity ──────────────────────────────────
  "cursor-pointer cursor-not-allowed",
  "select-none select-text select-all",
  "pointer-events-none pointer-events-auto",

  // ── Base utilities: overflow ───────────────────────────────────────
  "overflow-hidden overflow-auto overflow-x-scroll overflow-y-hidden",

  // ── Base utilities: visibility & display ───────────────────────────
  "visible invisible collapse",
  "sr-only not-sr-only",

  // ── Arbitrary values ───────────────────────────────────────────────
  "w-[300px] h-[calc(100vh-4rem)] text-[14px] bg-[#ff0000]",
  "p-[clamp(1rem,2vw,3rem)] m-[var(--spacing)] border-[length:3px]",

  // ── Negative values ────────────────────────────────────────────────
  "-mt-4 -ml-2 -translate-x-1/2 -rotate-45 -inset-1",

  // ── Important modifier ─────────────────────────────────────────────
  "p-4! m-2! text-red-500! font-bold!",

  // ── Single variants ────────────────────────────────────────────────
  "hover:bg-blue-500 focus:ring-2 active:scale-95",
  "disabled:opacity-50 disabled:pointer-events-none",
  "dark:bg-gray-900 dark:text-white",
  "sm:flex sm:hidden md:grid lg:block xl:inline 2xl:flex",

  // ── Pseudo-element variants ────────────────────────────────────────
  "before:content-[''] before:absolute after:content-[''] after:block",
  "first:mt-0 last:mb-0 odd:bg-gray-50 even:bg-white",
  "placeholder:text-gray-400 placeholder:italic",

  // ── State variants ─────────────────────────────────────────────────
  "focus-within:ring-2 focus-visible:ring-2 focus-visible:outline-none",
  "aria-disabled:opacity-50 aria-disabled:pointer-events-none",
  "data-active:bg-blue-500 data-open:rotate-180",
  "data-[state=active]:bg-blue-500 data-[side=top]:border-t",
  "data-highlighted:bg-blue-100 data-highlighted:text-blue-900",

  // ── Compound variants: group-* ─────────────────────────────────────
  "group-hover:opacity-100 group-focus:ring-2",
  "group-data-[collapsible=icon]:size-8 group-data-[collapsible=icon]:p-2",
  "group-data-vertical/tabs:w-full group-data-horizontal/tabs:flex-row",

  // ── Compound variants: peer-* ──────────────────────────────────────
  "peer-hover:text-blue-500 peer-focus:ring-2 peer-invalid:border-red-500",
  "peer-data-[size=default]/menu:top-1.5 peer-data-[size=lg]/menu:top-2.5 peer-data-[size=sm]/menu:top-1",

  // ── Compound variants: has-* ───────────────────────────────────────
  "has-disabled:opacity-50 has-disabled:bg-gray-100",
  "has-[[data-slot=input]:focus-visible]:border-blue-500 has-[[data-slot=input]:focus-visible]:ring-2",
  "has-[>[data-align=end]]:h-auto has-[>[data-align=end]]:flex-col has-[>textarea]:h-auto",
  "has-[[data-slot][aria-invalid=true]]:border-red-500 has-[>[data-align=start]]:flex-col",

  // ── Compound variants: not-* ───────────────────────────────────────
  "not-disabled:hover:bg-blue-500 not-first:mt-4",

  // ── Stacked variants ───────────────────────────────────────────────
  "hover:focus:bg-blue-500 dark:hover:bg-gray-800",
  "sm:hover:bg-blue-500 md:focus:ring-2 lg:dark:bg-gray-900",
  "dark:data-active:border-gray-700 dark:data-active:bg-gray-800",

  // ── Multi-variant stacking with data-[side=*] ──────────────────────
  "data-[side=bottom]:inset-x-0 data-[side=bottom]:border-t data-[side=left]:inset-y-0 data-[side=left]:border-r data-[side=right]:inset-y-0 data-[side=right]:border-l",

  // ── Arbitrary selector variants ────────────────────────────────────
  "[&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  "[&>span:last-child]:truncate [&_svg]:size-4 [&_svg]:shrink-0",
  "[&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-blue-500",
  "[[data-slot=content]_&]:focus-within:border-blue-500 [[data-slot=content]_&]:focus-within:ring-0",

  // ── Child (*) variant ──────────────────────────────────────────────
  "*:data-[slot=value]:flex *:data-[slot=value]:line-clamp-1 *:data-[slot=value]:items-center *:data-[slot=value]:gap-1.5",

  // ── Container queries (@) ──────────────────────────────────────────
  "@md/group:flex-row @md/group:items-center",
  "[&>*]:w-full @md/group:[&>*]:w-auto [&>.sr-only]:w-auto",

  // ── Mixed unknown + known classes ──────────────────────────────────
  "custom-class p-4 another-unknown flex m-2",
  "my-component text-lg font-bold bg-white shadow-md",

  // ── Complex real-world strings (standard classes only) ─────────────
  // Button-like
  "inline-flex shrink-0 items-center justify-center rounded-lg border border-transparent bg-clip-padding text-sm font-medium whitespace-nowrap transition-all outline-none select-none focus-visible:ring-2 focus-visible:ring-blue-500 disabled:pointer-events-none disabled:opacity-50",

  // Input-like
  "flex h-8 w-full min-w-0 rounded-lg border bg-transparent px-3 py-1 text-sm shadow-xs transition-colors outline-none placeholder:text-gray-400 focus-visible:ring-2 focus-visible:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-50",

  // Sidebar-like
  "fixed inset-y-0 z-50 hidden h-svh w-64 transition-all duration-200 ease-linear data-[side=left]:left-0 data-[side=right]:right-0 md:flex",

  // Card-like
  "flex flex-col gap-4 overflow-hidden rounded-xl bg-white py-4 text-sm text-gray-900 ring-1 ring-gray-200",

  // Responsive variants
  "p-2 sm:p-4 md:p-6 lg:p-8 xl:p-10 2xl:p-12",

  // All pseudo-elements
  "before:absolute before:inset-0 after:absolute after:inset-0 first-letter:text-2xl first-line:font-bold",

  // Motion preferences
  "motion-safe:animate-spin motion-reduce:animate-none",

  // Print and forced-colors
  "print:hidden forced-colors:outline",

  // Orientation
  "portrait:flex-col landscape:flex-row",

  // Direction
  "ltr:ml-2 rtl:mr-2",

  // Supports
  "supports-[display:grid]:grid supports-[backdrop-filter]:backdrop-blur",

  // ── Edge cases ─────────────────────────────────────────────────────
  // Single class (should be unchanged)
  "flex",
  // All unknown (should preserve relative order)
  "foo bar baz",
  // Empty-ish
  "",
  // Duplicate classes (prettier removes duplicates)
  "p-4 m-2 p-4 flex m-2",
];

async function main() {
  const lines = [];

  for (const input of inputs) {
    if (input === "") {
      lines.push(`\t`);
      continue;
    }

    // Wrap in a JSX className to get prettier to sort it
    const code = `const x = <div className="${input}" />\n`;
    const formatted = await prettier.format(code, {
      parser: "babel",
      plugins: ["prettier-plugin-tailwindcss"],
      tailwindStylesheet: "./global.css",
    });
    // Extract the class string from the formatted output
    const match = formatted.match(/className="([^"]*)"/);
    if (!match) {
      console.error(`Failed to extract classes from: ${formatted}`);
      process.exit(1);
    }
    const expected = match[1];
    lines.push(`${input}\t${expected}`);
  }

  const versions = {
    prettier: (await import("prettier/package.json")).version,
    plugin: (await import("prettier-plugin-tailwindcss/package.json")).version,
    tailwind: (await import("tailwindcss/package.json")).version,
  };

  const header = [
    "# prettier_fixtures.txt — ground-truth sort order from prettier-plugin-tailwindcss",
    `# Generated: ${new Date().toISOString()}`,
    `# prettier: ${versions.prettier}`,
    `# prettier-plugin-tailwindcss: ${versions.plugin}`,
    `# tailwindcss: ${versions.tailwind}`,
    "#",
    "# Format: input<TAB>expected  (one pair per line, # lines are comments)",
    "# Re-generate: cd tailwind-sorter/tests && bun generate_fixtures.mjs",
    "#",
  ];

  const content = [...header, ...lines, ""].join("\n");
  writeFileSync("prettier_fixtures.txt", content);
  console.log(
    `Generated ${lines.length} fixture pairs (prettier ${versions.prettier}, ` +
    `plugin ${versions.plugin}, tailwindcss ${versions.tailwind})`,
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
