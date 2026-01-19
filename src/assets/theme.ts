import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

/**
 * shadcn/ui inspired dark theme using Tailwind Slate palette
 * Based on Figma: https://www.figma.com/community/file/1055912623219285224
 */
export const AuraPlus = definePreset(Aura, {
  semantic: {
    // Teal accent color (matches shadcn avatar in Figma)
    // primary: {
    //   50: '#f0fdfa',
    //   100: '#ccfbf1',
    //   200: '#99f6e4',
    //   300: '#5eead4',
    //   400: '#2dd4bf',
    //   500: '#14b8a6',
    //   600: '#0d9488',
    //   700: '#0f766e',
    //   800: '#115e59',
    //   900: '#134e4a',
    //   950: '#042f2e',
    // },
    colorScheme: {
      light: {
        // Light mode uses standard slate
        // surface: {
        //   0: '#ffffff',
        //   50: '#f8fafc',
        //   100: '#f1f5f9',
        //   200: '#e2e8f0',
        //   300: '#cbd5e1',
        //   400: '#94a3b8',
        //   500: '#64748b',
        //   600: '#475569',
        //   700: '#334155',
        //   800: '#1e293b',
        //   900: '#0f172a',
        //   950: '#020817',
        // },
      },
      dark: {
        // Dark mode - shadcn/ui slate palette
        // surface: {
        //   0: '#f8fafc',    // slate-50 - brightest text
        //   50: '#f1f5f9',   // slate-100
        //   100: '#e2e8f0',  // slate-200
        //   200: '#cbd5e1',  // slate-300
        //   300: '#94a3b8',  // slate-400 - muted text
        //   400: '#64748b',  // slate-500
        //   500: '#475569',  // slate-600
        //   600: '#334155',  // slate-700 - borders, dividers
        //   700: '#1e293b',  // slate-800 - card background
        //   800: '#0f172a',  // slate-900 - elevated surface
        //   900: '#020817',  // slate-950 - main background
        //   950: '#010409',  // even deeper
        // },
        primary: {
          color: '#2dd4bf',           // teal-400
          contrastColor: '#042f2e',   // teal-950
          hoverColor: '#14b8a6',      // teal-500
          activeColor: '#0d9488',     // teal-600
        },
        highlight: {
          background: 'rgba(45, 212, 191, 0.16)',  // teal with opacity
          focusBackground: 'rgba(45, 212, 191, 0.24)',
          color: '#2dd4bf',
          focusColor: '#5eead4',
        },
      },
    },
  },
});
