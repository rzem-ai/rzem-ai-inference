import type { InputGroupDesignTokens } from '@primeuix/themes/types/inputgroup';

export const inputgroup: InputGroupDesignTokens = {
  addon: {
    borderRadius: '{border.radius.md}',
    padding: '0.1rem',
    minWidth: '2.5rem',
  },
  colorScheme: {
    light: {
      addon: {
        background: '{form.field.background}',
        borderColor: '{form.field.border.color}',
        color: '{form.field.icon.color}',
      },
    },
    dark: {
      addon: {
        background: '{form.field.background}',
        borderColor: '{form.field.border.color}',
        color: '{form.field.icon.color}',
      },
    },
  },
};
