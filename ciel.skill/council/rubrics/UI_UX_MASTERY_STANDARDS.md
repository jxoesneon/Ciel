# UI/UX MASTERY STANDARDS — Design Council Master Specifications (2026)

Synthesized by a joint session of the **Ciel System Council of Five** and the **Design Council of Five**.

---

## The Top 10 UI/UX Engineering & Design Standards

### 1. Agent UX & Oversight Layer
- **Plan-Execute Transparency**: Before executing multi-step tasks, render the agent's proposed plan clearly for user review and course-correction.
- **Tool Invocation State**: Expose real-time progress indicators when external tools or APIs are running to reduce cognitive uncertainty.
- **Human-in-the-Loop Interception**: High-risk or destructive actions MUST require explicit user approval.

### 2. Core Web Vitals (CWV) Performance Thresholds
- **LCP (Largest Contentful Paint)**: $< 2.5\text{s}$ (hero image & main text loading efficiency).
- **INP (Interaction to Next Paint)**: $< 200\text{ms}$ (main thread responsiveness, zero laggy taps/clicks).
- **CLS (Cumulative Layout Shift)**: $< 0.1$ (visual stability, reserved aspect ratio boxes for async elements).

### 3. WCAG 2.2 Level AA/AAA Accessibility (Inclusion Lens Veto)
- **Focus Appearance (3.2.6)**: Highly visible, high-contrast keyboard focus rings ($\ge 3:1$ contrast ratio).
- **Target Size (2.5.8)**: Interactive elements MUST meet minimum $24 \times 24\text{px}$ touch targets.
- **Consistent Navigation**: Fixed layout regions for help, navigation, and error feedback.
- **Non-Cognitive Auth**: Authentication flows must offer alternatives to complex memory or visual puzzles.

### 4. Bento Grid Layout Architecture
- Modular, compartmentalized grid systems for complex dashboards and SaaS applications.
- Card-based spatial hierarchy inspired by clean grid dynamics; balances visual density with breathing space.

### 5. Liquid Glass & Tactile Depth
- Layered backdrop-blur filters (`backdrop-filter: blur(...)`), subtle glass borders (`border: 1px solid rgba(255,255,255,0.1)`), and translucent surfaces.
- Elevates perceived product trust without compromising readability or background contrast.

### 6. Tailwind CSS v4 & Native Tokens
- CSS-first configuration using `@theme` and native CSS custom properties.
- Lightning-fast Rust-based build engine; zero `tailwind.config.js` overhead; native container query support.

### 7. shadcn/ui & Headless Accessible Primitives
- Full code ownership model (copy-paste components built on Radix UI primitives).
- Unrestricted styling control with zero black-box vendor lock-in.

### 8. Purposeful Micro-Interactions
- Tactile button ripples, smooth state transitions, hover lift effects, and active state feedback.
- Every micro-animation MUST serve a functional feedback purpose, confirming state changes within $< 100\text{ms}$.

### 9. Dark Mode & Contrast Engineering
- Intentional HSL color scales; elevated surfaces represented by lighter surface tones rather than higher saturation.
- Pure dark background (`#09090b` / `#000000`) with high-contrast text (`#f4f4f5`) preventing eye fatigue.

### 10. Design System Governance & Token Consistency
- Strict adherence to predefined token systems for spacing (`gap-2`, `gap-4`, `p-6`), typography, and rounded borders (`rounded-xl`).
- Rejection of ad-hoc, hardcoded static offsets in favor of dynamic container math.

---

## Design Council Scoring Rubric Integration

Each UI/UX artifact submitted to Ciel is evaluated across the 5 Design Council lenses using these standards:
- **Clarity Lens**: Evaluates Bento Grid structure, typography, and visual hierarchy.
- **Inclusion Lens**: Enforces WCAG 2.2 standards. Any score $\le 3.0$ triggers an absolute veto.
- **Efficiency Lens**: Enforces Core Web Vitals (INP $< 200\text{ms}$, LCP $< 2.5\text{s}$) and ergonomics.
- **Aesthetics Lens**: Evaluates Liquid Glass, dark mode scales, typography, and perceived trust.
- **Actionability Lens**: Evaluates Agent UX transparency, micro-interactions, and conversion affordance.
