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
        background: 'transparent',
        borderColor: '{surface.700}',
        color: '{surface.400}',
      },
    },
    dark: {
      addon: {
        background: 'transparent',
        borderColor: '{surface.700}',
        color: '{surface.400}',
      },
    },
  },
};
