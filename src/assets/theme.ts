import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

export const AuraPlus = definePreset(Aura, {
  semantic: {
    primary: {
      50: '{blue.50}',
      100: '{blue.100}',
      200: '{blue.200}',
      300: '{blue.300}',
      400: '{blue.400}',
      500: '{blue.500}',
      600: '{blue.600}',
      700: '{blue.700}',
      800: '{blue.800}',
      900: '{blue.900}',
      950: '{blue.950}',
    },
    colorScheme: {
      light: {
        gray: {
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
      },
      dark: {
        gray: {
          0: '#ffffff',
          100: '#383838',
          200: '#363636',
          300: '#333333',
          400: '#2E2E2E',
          500: '#2C2C2C',
          600: '#272727',
          700: '#252525',
          800: '#232323',
          900: '#1E1E1E',
          950: '#121212',
        }
      },
    },
  },
});
