import type { ToggleButtonDesignTokens } from '@primeuix/themes/types/togglebutton';

export const togglebutton: ToggleButtonDesignTokens = {
  root: {
    padding: '0.25rem',
    borderRadius: '{border.radius.md}',
    gap: '0.5rem',
    fontWeight: '500',
    focusRing: {
      width: '{focus.ring.width}',
      style: '{focus.ring.style}',
      offset: '{focus.ring.offset}',
      shadow: '{focus.ring.shadow}',
    },
    transitionDuration: '{form.field.transition.duration}',
    sm: {
      fontSize: '{form.field.sm.font.size}',
      padding: '0.25rem',
    },
    lg: {
      fontSize: '{form.field.lg.font.size}',
      padding: '0.25rem',
    },
  },
  content: {
    padding: '0.25rem 0.25rem',
    borderRadius: '{border.radius.sm}',
    checkedShadow: 'none',
    sm: {
      padding: '0.15rem',
    },
    lg: {
      padding: '0.25rem 0.75rem',
    },
  },
  icon: {},
  colorScheme: {
    light: {
      root: {
        background: '{surface.900}',
        borderColor: '{surface.600}',
        checkedBackground: '{surface.800}',
        checkedBorderColor: '{surface.600}',
        checkedColor: '{surface.950}',
        color: '{surface.100}',
        disabledBackground: '{form.field.disabled.background}',
        disabledBorderColor: '{form.field.disabled.background}',
        disabledColor: '{form.field.disabled.color}',
        hoverBackground: '{surface.950}',
        hoverColor: '{surface.100}',
        invalidBorderColor: '{form.field.invalid.border.color}',
        focusRing: {
          color: '{focus.ring.color}',
        },
      },
      content: {
        checkedBackground: '{blue.500}',
      },
      icon: {
        color: '{surface.400}',
        hoverColor: '{surface.200}',
        checkedColor: '{surface.0}',
      },
    },
    dark: {
      root: {
        background: '{surface.900}',
        borderColor: '{surface.900}',
        checkedBackground: '{surface.900}',
        checkedBorderColor: '{surface.900}',
        checkedColor: '{surface.0}',
        color: '{surface.100}',
        disabledBackground: '{form.field.disabled.background}',
        disabledBorderColor: '{form.field.disabled.background}',
        disabledColor: '{form.field.disabled.color}',
        hoverBackground: '{surface.950}',
        hoverColor: '{surface.100}',
        invalidBorderColor: '{form.field.invalid.border.color}',
      },
      content: {
        padding: '0.25rem 0.75rem',
        borderRadius: '{content.border.radius}',
        checkedBackground: '{surface.500}',
        sm: {
          padding: '0.25rem 0.75rem',
        },
        lg: {
          padding: '0.25rem 0.75rem',
        },
      },
      icon: {
        color: '{surface.400}',
        hoverColor: '{surface.200}',
        checkedColor: '{surface.0}',
      },
    },
  },
};
