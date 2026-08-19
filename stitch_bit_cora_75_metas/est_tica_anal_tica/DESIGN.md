---
name: Estética Analítica
colors:
  surface: '#f9f9fe'
  surface-dim: '#d9dadf'
  surface-bright: '#f9f9fe'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#f3f3f8'
  surface-container: '#ededf3'
  surface-container-high: '#e7e8ed'
  surface-container-highest: '#e2e2e7'
  on-surface: '#191c1f'
  on-surface-variant: '#42474f'
  inverse-surface: '#2e3034'
  inverse-on-surface: '#f0f0f5'
  outline: '#727780'
  outline-variant: '#c2c7d1'
  surface-tint: '#2b6197'
  primary: '#1b558b'
  on-primary: '#ffffff'
  primary-container: '#3a6ea5'
  on-primary-container: '#e4eeff'
  inverse-primary: '#9fcaff'
  secondary: '#5f5e5d'
  on-secondary: '#ffffff'
  secondary-container: '#e2dfdd'
  on-secondary-container: '#636261'
  tertiary: '#6e4d00'
  on-tertiary: '#ffffff'
  tertiary-container: '#8d6404'
  on-tertiary-container: '#ffebce'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#d2e4ff'
  primary-fixed-dim: '#9fcaff'
  on-primary-fixed: '#001c37'
  on-primary-fixed-variant: '#02497e'
  secondary-fixed: '#e5e2e0'
  secondary-fixed-dim: '#c9c6c4'
  on-secondary-fixed: '#1c1c1a'
  on-secondary-fixed-variant: '#474745'
  tertiary-fixed: '#ffdea9'
  tertiary-fixed-dim: '#f3be5e'
  on-tertiary-fixed: '#271900'
  on-tertiary-fixed-variant: '#5e4100'
  background: '#f9f9fe'
  on-background: '#191c1f'
  surface-variant: '#e2e2e7'
typography:
  hero-num:
    fontFamily: Inter
    fontSize: 56px
    fontWeight: '600'
    lineHeight: 64px
  card-value:
    fontFamily: Inter
    fontSize: 32px
    fontWeight: '600'
    lineHeight: 40px
  section-title:
    fontFamily: Inter
    fontSize: 18px
    fontWeight: '600'
    lineHeight: 24px
  body:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-muted:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: 0.04em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  base: 8px
  card-padding: 20px
  card-gap: 16px
  sidebar-width: 200px
  titlebar-height: 44px
---

## Brand & Style
The design system focuses on a high-density, analytical desktop environment tailored for professional workflows. The personality is restrained, authoritative, and strictly utilitarian, evoking the feel of a precision instrument. 

The aesthetic leverages a **Minimalist** approach with a **Warm Neutral** foundation. By avoiding shadows and relying on hairline borders and subtle tonal shifts, the interface ensures that the user's focus remains entirely on data integrity and information hierarchy. All interface text is presented in **Spanish**.

## Colors
The palette is built on a "Warm Greyscale" to reduce eye strain during long-duration usage. 
- **Surface Strategy:** Layers are defined by value rather than shadow. Use `#f4f4f2` for the base canvas, `#fbfbfa` for content panels, and `#ffffff` for modal-level elements. Use `#eaeae7` for "sunken" states like empty wells or active navigation backgrounds.
- **Accents:** Slate blue (`#3a6ea5`) is used exclusively for functional action points, focus indicators, and primary data visualization.
- **Status:** Semantic colors (Green, Amber, Red) must always be accompanied by labels or icons to ensure accessibility within the dense UI.

## Typography
The system uses **Inter** exclusively. To support data density and financial/technical reporting, **Tabular Numerals** (`tnum`) must be enabled for all numerical displays to ensure vertical alignment in tables and dashboards.

- **Hero Numbers:** Reserved for top-level KPIs.
- **Muted Labels:** Used for table headers, metadata descriptions, and category tags.
- **Line Heights:** Tightened to maximize vertical information density without sacrificing legibility.

## Layout & Spacing
This is a **Fixed-Fluid Hybrid** layout optimized for 1440px+ displays.
- **App Shell:** A 200px persistent left sidebar (`#fbfbfa`) is separated by a 1px right border (`#dcdbd7`). The top title bar is fixed at 44px.
- **Grid:** An 8px base unit governs all dimensions.
- **Containers:** Standard content cards utilize 20px of internal padding with a 16px gap between adjacent modules.
- **Density:** Elements should be tightly packed. Use the 8px grid to minimize white space between related data points while maintaining clear structural separation via hairline borders.

## Elevation & Depth
Depth is strictly two-dimensional. **No drop shadows or blurs are permitted.**
- **Hairline Borders:** Use 1px solid `#dcdbd7` to define all boundaries (cards, inputs, dividers).
- **Z-axis Logic:** Use color shifts to indicate hierarchy. The "Sunken" state (`#eaeae7`) is used for the active navigation track and secondary input backgrounds. The "Elevated" state (`#ffffff`) is reserved for the primary data area or overlay panels.
- **Focus:** Primary focus is indicated by a 2px solid stroke of Slate Blue (`#3a6ea5`) or a subtle 1px inset border.

## Shapes
The shape language is conservative and geometric.
- **Cards & Inputs:** 6px radius to provide a subtle professional softness.
- **Buttons & Chips:** 4px radius for a more rigid, functional appearance.
- **Constraint:** "Pill" shapes or fully rounded circles are prohibited, except for user avatars. All interactive triggers must maintain their rectangular integrity.

## Components
- **Buttons:** Primary buttons use a solid Slate Blue fill with white text. Secondary buttons use a white background with a 1px border. 4px corner radius.
- **Data Tables:** Row heights should be 32px or 36px. Use horizontal borders only. Alternate row striping is not used; use hover states to highlight rows.
- **Navigation:** The active state in the sidebar is marked by a `#eaeae7` background and a 3px vertical Slate Blue bar on the far left edge.
- **Input Fields:** 6px radius. Background is white, border is hairline. On focus, the border changes to Slate Blue. Labels are always positioned above the field using the "body" or "label-muted" style.
- **Chips:** Used for filtering and status. 4px radius. Backgrounds should be very light tints of the status color or neutral greys.
- **Cards:** 6px radius, `#ffffff` background, 1px border, 20px internal padding. Title bar within cards should be separated by a hairline divider.