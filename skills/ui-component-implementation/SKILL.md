---
name: ui-component-implementation
version: 1.0.0
format: skill/1.0
description: CIEL's framework for implementing accessible UI components with shadcn/ui, Radix primitives, and Tailwind utilities. Advisory and code-generation only.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:design"]
triggers:
  - pattern: "(build|implement|add).*(shadcn|radix|component|dialog|dropdown|form|table)"
    confidence: 0.9
  - pattern: "(tailwind|dark mode|responsive).*(layout|styling|theme)"
    confidence: 0.9
source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---

# CIEL ADAPTATION: UI Component Implementation

This skill provides the *implementation* layer for UI work — the `ui-ux-design` skill covers guidelines and style selection; this one covers how to actually build accessible components with shadcn/ui (Radix UI primitives + Tailwind CSS). It is advisory and code-generation only: it emits code patterns, never executes them.

## Core Stack

- **shadcn/ui**: Copy-paste components built on Radix UI primitives; TypeScript-first; CLI-installed (`npx shadcn@latest init`, `npx shadcn@latest add button card dialog`).
- **Tailwind CSS**: Utility-first, build-time, zero runtime. Mobile-first responsive. Automatic dead-code elimination.
- **Radix UI**: Unstyled accessibility primitives (focus traps, ARIA, keyboard nav) that shadcn/ui wraps.

## Component Patterns

- **Composition over configuration**: Build complex UIs from composable primitives (`Card` = `Card`/`CardHeader`/`CardTitle`/`CardContent`).
- **Variant props**: Use `cva` (class-variance-authority) for `variant`/`size` props; never branch on inline conditionals.
- **Form validation**: `react-hook-form` + `zod` resolver + shadcn `Form`/`FormField`/`FormItem`/`FormControl`/`FormMessage`. Schema is single source of truth.
- **Overlays**: Dialog, Drawer, Popover, Toast, Command — all Radix-backed; rely on their focus management, do not reimplement.
- **Data display**: Table, Data Table (TanStack), Avatar, Badge, Skeleton — prefer these over hand-rolled markup.

## Theming & Dark Mode

- **CSS variables in `globals.css`**: shadcn uses HSL channel triplets (`--primary: 222.2 47.4% 11.2%`) consumed via `hsl(var(--primary))`.
- **Token set**: `--background`, `--foreground`, `--primary`, `--secondary`, `--muted`, `--accent`, `--destructive`, `--border`, `--input`, `--ring`, `--radius`.
- **Dark mode**: Override the same variables under `.dark`. Use `next-themes` (`attribute="class"`, `defaultTheme="system"`, `suppressHydrationWarning` on `<html>`). Toggle exposes `sr-only` label for screen readers.
- **Never hardcode hex** in components — always reference a token so theme switching works.

## Tailwind Utility Patterns

- **Layout**: `flex`, `grid` (`grid-cols-1 md:grid-cols-2 lg:grid-cols-3`), `gap-{n}`, `container mx-auto`.
- **Spacing**: 4px base scale (`p-4`, `gap-6`, `space-y-4`). Keep to the scale; use arbitrary values `[17rem]` only as a last resort.
- **Typography**: `text-{sm|base|lg|xl|2xl}`, `font-{medium|semibold|bold}`, `leading-tight|normal`, `tracking-tight`.
- **State**: `hover:`, `focus-visible:`, `active:`, `disabled:`. Always include `focus-visible:` for keyboard users.
- **Customization**: `@theme` directive (Tailwind v4) or `tailwind.config` extensions for custom colors/fonts/spacing. Extract repeated class clusters into components via `@apply` only for genuine reuse.

## Responsive Layout

- **Mobile-first**: Base styles target mobile; layer `sm`(640)→`md`(768)→`lg`(1024)→`xl`(1280)→`2xl`(1536) variants upward.
- **Container queries** (`@container`) for component-level responsiveness independent of viewport.
- **Touch targets**: minimum 44×44pt; use `size-10`/`size-11` for icon buttons.

## Accessibility

- **Radix primitives give you**: focus traps, `aria-*`, roving tabindex, ESC handling, scroll lock — for free. Don't override unless you understand the contract.
- **Keyboard nav**: Every interactive element must be reachable and operable by keyboard; visible `focus-visible` ring.
- **Semantic HTML**: Use native elements (`button`, `nav`, `main`, `section`) before ARIA; ARIA only when no native element fits.
- **Screen readers**: `sr-only` for labels on icon-only controls; `aria-live="polite"` for toasts/dynamic updates.
- **Forms**: Associate `<label>` with input; surface errors via `FormMessage` with `role="alert"` semantics.

## Anti-Patterns

- **Hardcoded hex in components**: Breaks theming and dark mode — always use tokens.
- **Dynamic class names**: `className={`bg-${color}-500`}` defeats Tailwind's purge — use static classes or `cva` variants.
- **Reimplementing overlays**: Hand-built modals/dropdowns miss focus traps and ARIA — use Radix-backed shadcn components.
- **Skipping focus-visible**: Removing outline without a replacement ring abandons keyboard users.
- **Emoji as icons**: Use `lucide-react` SVG icons for structural/navigation affordances.
- **Per-screen styling**: Duplicating styles per route instead of composing shared components and tokens.
