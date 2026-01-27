import type { ProgressBarDesignTokens } from '@primeuix/themes/types/progressbar';

export const progressbar: ProgressBarDesignTokens = {
  root: {
    background: '{surface.900}',
    borderRadius: '{content.border.radius}',
    height: '1rem',
  },
  label: {
    color: '{primary.contrast.color}',
    fontSize: '0.75rem',
    fontWeight: '600',
  },
  value: {
    background: '{blue.500}',
  },
};
