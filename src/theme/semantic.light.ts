import type { AuraBaseTokenSections } from '@primeuix/themes/aura/base';

type ColorSchemeLight = NonNullable<
  NonNullable<AuraBaseTokenSections.Semantic['colorScheme']>['light']
>;

export const semanticLight: ColorSchemeLight = {
  surface: {
    0: '#000000',
    50: '{gray.950}',
    100: '{gray.900}',
    200: '{gray.800}',
    300: '{gray.700}',
    400: '{gray.600}',
    500: '{gray.500}',
    600: '{gray.400}',
    700: '{gray.300}',
    800: '{gray.200}',
    900: '{gray.100}',
    950: '{gray.50}',
  },
  primary: {
    color: '{primary.400}',
    contrastColor: '#ffffff',
    hoverColor: '{primary.500}',
    activeColor: '{primary.500}',
  },
  highlight: {
    background: '{primary.50}',
    focusBackground: '{primary.100}',
    color: '{primary.700}',
    focusColor: '{primary.800}',
  },
  content: {
    background: '{surface.900}',
    borderColor: '{surface.700}',
    color: '{text.color}',
    hoverBackground: '{surface.800}',
    hoverColor: '{text.hover.color}',
  },
  text: {
    color: '{surface.400}',
    hoverColor: '{surface.800}',
    mutedColor: '{surface.500}',
    hoverMutedColor: '{surface.600}',
  },
  formField: {
    background: '{surface.900}',
    borderColor: '{surface.600}',
    color: '{surface.500}',
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
  overlay: {
    select: {
      background: '{surface.0}',
      borderColor: '{surface.200}',
      color: '{text.color}',
    },
    popover: {
      background: '{surface.0}',
      borderColor: '{surface.200}',
      color: '{text.color}',
    },
    modal: {
      background: '{surface.800}',
      borderColor: '{surface.950}',
      color: '{text.color}',
    },
  },
};
