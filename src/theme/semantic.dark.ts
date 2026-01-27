import type { AuraBaseTokenSections } from '@primeuix/themes/aura/base';

type ColorSchemeDark = NonNullable<
  NonNullable<AuraBaseTokenSections.Semantic['colorScheme']>['dark']
>;

export const semanticDark: ColorSchemeDark = {
  surface: {
    0: '#ffffff',
    50: '{gray.50}',
    100: '{gray.100}',
    200: '{gray.200}',
    300: '{gray.300}',
    400: '{gray.400}',
    500: '{gray.500}',
    600: '{gray.600}',
    700: '{gray.700}',
    800: '{gray.800}',
    900: '{gray.900}',
    950: '{gray.950}',
  },
  primary: {
    activeColor: '{red.500}',
    color: '{blue.500}',
    contrastColor: '{surface.100}',
    hoverColor: '{blue.600}',
  },
  content: {
    background: '{surface.900}',
    borderColor: '{surface.700}',
    color: '{text.color}',
    hoverBackground: '{surface.800}',
    hoverColor: '{text.hover.color}',
  },
  text: {
    color: '{surface.0}',
    hoverColor: '{surface.0}',
    mutedColor: '{surface.400}',
    hoverMutedColor: '{surface.300}',
  },
  formField: {
    background: '{surface.900}',
    borderColor: '{surface.500}',
    color: '{surface.0}',
    disabledBackground: '{surface.700}',
    disabledColor: '{surface.400}',
    filledBackground: '{surface.800}',
    filledFocusBackground: '{surface.800}',
    filledHoverBackground: '{surface.800}',
    floatLabelActiveColor: '{surface.400}',
    floatLabelColor: '{surface.400}',
    floatLabelFocusColor: '{primary.color}',
    floatLabelInvalidColor: '{form.field.invalid.placeholder.color}',
    focusBorderColor: '{primary.color}',
    hoverBorderColor: '{surface.500}',
    iconColor: '{surface.400}',
    invalidBorderColor: '{red.300}',
    invalidPlaceholderColor: '{red.400}',
    placeholderColor: '{surface.400}',
    shadow: '0 0 #0000, 0 0 #0000, 0 1px 2px 0 rgba(18, 18, 23, 0.05)',
  },
  highlight: {
    background: '{red.500}',
    color: '{red.500}',
    focusBackground: '{red.500}',
    focusColor: '{red.500}',
  },
  list: {
    option: {
      color: '{text.color}',
      focusBackground: '{surface.800}',
      focusColor: '{text.hover.color}',
      selectedBackground: '{highlight.background}',
      selectedColor: '{highlight.color}',
      selectedFocusBackground: '{highlight.focus.background}',
      selectedFocusColor: '{highlight.focus.color}',
      icon: {
        color: '{surface.500}',
        focusColor: '{surface.400}',
      },
    },
    optionGroup: {
      background: 'transparent',
      color: '{text.muted.color}',
    },
  },
  mask: {
    background: 'rgba(0,0,0,0.6)',
    color: '{surface.200}',
  },
  navigation: {
    item: {
      activeBackground: '{surface.800}',
      activeColor: '{text.hover.color}',
      color: '{text.color}',
      focusBackground: '{surface.800}',
      focusColor: '{text.hover.color}',
      icon: {
        color: '{surface.500}',
        focusColor: '{surface.400}',
        activeColor: '{surface.400}',
      },
    },
    submenuIcon: {
      activeColor: '{surface.400}',
      color: '{surface.500}',
      focusColor: '{surface.400}',
    },
    submenuLabel: {
      background: 'transparent',
      color: '{text.muted.color}',
    },
  },
  overlay: {
    select: {
      background: '{surface.700}',
      borderColor: '{surface.400}',
      color: '{text.color}',
    },
    popover: {
      background: '{surface.900}',
      borderColor: '{surface.700}',
      color: '{text.color}',
    },
    modal: {
      background: '{surface.900}',
      borderColor: '{surface.700}',
      color: '{text.color}',
    },
  },
};