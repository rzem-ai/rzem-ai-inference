import type { DialogDesignTokens } from '@primeuix/themes/types/dialog';

export const dialog: DialogDesignTokens = {
  root: {
    background: '{overlay.modal.background}',
    borderColor: '{overlay.modal.border.color}',
    color: '{overlay.modal.color}',
    borderRadius: '{overlay.modal.border.radius}',
    shadow: '{overlay.modal.shadow}',
  },
  colorScheme: {
    light: {
      root: {
        background: '{overlay.modal.background}',
        borderColor: '{overlay.modal.border.color}',
        color: '{overlay.modal.color}',
        borderRadius: '{overlay.modal.border.radius}',
        shadow: '{overlay.modal.shadow}',
      },
      content: {},
      footer: {},
      header: {},
      title: {},
    },
    dark: {
      root: {
        background: '{overlay.modal.background}',
        borderColor: '{overlay.modal.border.color}',
        color: '{overlay.modal.color}',
        borderRadius: '{overlay.modal.border.radius}',
        shadow: '{overlay.modal.shadow}',
      },
      content: {},
      footer: {},
      header: {},
      title: {},
    },
  },
  content: {
    padding: '0 {overlay.modal.padding} {overlay.modal.padding} {overlay.modal.padding}',
  },
  footer: {
    padding: '0 {overlay.modal.padding} {overlay.modal.padding} {overlay.modal.padding}',
    gap: '0.5rem',
  },
  header: {
    padding: '{overlay.modal.padding}',
    gap: '0.5rem',
  },
  title: {
    fontSize: '1.25rem',
    fontWeight: '600',
  },
};
