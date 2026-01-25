import type { SliderDesignTokens } from '@primeuix/themes/types/slider';

export const slider: SliderDesignTokens = {
  root: {
    transitionDuration: '{transition.duration}',
  },
  handle: {
    width: '20px',
    height: '16px',
    borderRadius: '{border.radius.md}',
    content: {
      borderRadius: '{border.radius.sm}',
      width: '16px',
      height: '10px',
      shadow: '0',
    },
    focusRing: {
      width: '0',
      style: '0',
      offset: '0',
    },
  },
  range: {},
  track: {
    borderRadius: '{border.radius.md}',
    size: '5px',
  },
  colorScheme: {
    light: {
      handle: {
        background: '{blue.300}',
        hoverBackground: '{blue.500}',
        content: {
          background: '{blue.400}',
          hoverBackground: '{blue.500}',
          //shadow: '0px 1px 1px 0px rgba(0, 0, 0, 0.2)',
          shadow: '0px',
        },
        focusRing: {
          shadow: '0',
        },
      },
      range: {
        background: '{blue.400}',
      },
      track: {
        background: '{surface.700}',
      },
    },
    dark: {
      handle: {
        background: '{content.border.color}',
        hoverBackground: '{content.border.color}',
        content: {
          hoverBackground: '{content.background}',
          shadow: '0px 0.5px 0px 0px rgba(0, 0, 0, 0.08), 0px 1px 1px 0px rgba(0, 0, 0, 0.14)',
        },
        focusRing: {
          color: '{focus.ring.color}',
          shadow: '{focus.ring.shadow}',
        },
      },
      range: {
        background: '{primary.color}',
      },
      track: {
        background: '{content.border.color}',
      },
    },
  },
};
