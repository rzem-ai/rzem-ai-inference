import type { ToolbarDesignTokens } from '@primeuix/themes/types/toolbar';

export const toolbar: ToolbarDesignTokens = {
  root: {
    gap: '0.5rem',
    padding: '0.25rem',
  },

  colorScheme: {
    light: {
      root: {
        background: '{surface.800}',
        borderColor: '{surface.700}',
        borderRadius: '{content.border.radius}',
        color: '{surface.200}',
      },
    },
    dark: {
      root: {
        background: '{content.background}',
        borderColor: '{content.border.color}',
        borderRadius: '{content.border.radius}',
        color: '{content.color}',
      },
    },
  },
};
