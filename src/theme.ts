import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

/**
 * shadcn/ui inspired dark theme using Tailwind Slate palette
 * Based on Figma: https://www.figma.com/community/file/1055912623219285224
 */
export const AuraPlus = definePreset(Aura, {
  primitive: {
    gray: {
      50: '#fafafa',
      100: '#f5f5f5',
      200: '#e5e5e5',
      300: '#d4d4d4',
      400: '#a3a3a3',
      500: '#737373',
      600: '#525252',
      700: '#404040',
      800: '#262626',
      900: '#171717',
      950: '#0a0a0a',
    },
  },
  semantic: {
    transitionDuration: '0.2s',
    focusRing: {
      width: '1px',
      style: 'solid',
      color: '{primary.color}',
      offset: '2px',
      shadow: 'none',
    },
    disabledOpacity: '0.6',
    iconSize: '1rem',
    anchorGutter: '2px',
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
    formField: {
      paddingX: '0.75rem',
      paddingY: '0.5rem',
      sm: {
        fontSize: '0.875rem',
        paddingX: '0.625rem',
        paddingY: '0.375rem',
      },
      lg: {
        fontSize: '1.125rem',
        paddingX: '0.875rem',
        paddingY: '0.625rem',
      },
      borderRadius: '{border.radius.md}',
      focusRing: {
        width: '0',
        style: 'none',
        color: 'transparent',
        offset: '0',
        shadow: 'none',
      },
      transitionDuration: '{transition.duration}',
    },
    list: {
      padding: '0.25rem 0.25rem',
      gap: '2px',
      header: {
        padding: '0.5rem 1rem 0.25rem 1rem',
      },
      option: {
        padding: '0.5rem 0.75rem',
        borderRadius: '{border.radius.sm}',
      },
      optionGroup: {
        padding: '0.5rem 0.75rem',
        fontWeight: '600',
      },
    },
    content: {
      borderRadius: '{border.radius.md}',
    },
    mask: {
      transitionDuration: '0.3s',
    },
    navigation: {
      list: {
        padding: '0.25rem 0.25rem',
        gap: '2px',
      },
      item: {
        padding: '0.5rem 0.75rem',
        borderRadius: '{border.radius.sm}',
        gap: '0.5rem',
      },
      submenuLabel: {
        padding: '0.5rem 0.75rem',
        fontWeight: '600',
      },
      submenuIcon: {
        size: '0.875rem',
      },
    },
    overlay: {
      select: {
        borderRadius: '{border.radius.md}',
        shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
      },
      popover: {
        borderRadius: '{border.radius.md}',
        padding: '0.75rem',
        shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
      },
      modal: {
        borderRadius: '{border.radius.xl}',
        padding: '1.25rem',
        shadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1)',
      },
      navigation: {
        shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
      },
    },
    colorScheme: {
      dark: {
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
          color: '{blue.500}',
          contrastColor: '{gray.100}',
          hoverColor: '{blue.600}',
          activeColor: '{sky.500}',
        },
        highlight: {
          background: '{red.500}',
          focusBackground: '{red.500}',
          color: '{red.500}',
          focusColor: '{red.500}',
        },
        mask: {
          background: 'rgba(0,0,0,0.6)',
          color: '{gray.200}',
        },
        formField: {
          background: '{gray.700}',
          disabledBackground: '{gray.700}',
          filledBackground: '{gray.800}',
          filledHoverBackground: '{gray.800}',
          filledFocusBackground: '{gray.800}',
          borderColor: '{gray.500}',
          hoverBorderColor: '{gray.500}',
          focusBorderColor: '{primary.color}',
          invalidBorderColor: '{red.300}',
          color: '{gray.0}',
          disabledColor: '{gray.400}',
          placeholderColor: '{gray.400}',
          invalidPlaceholderColor: '{red.400}',
          floatLabelColor: '{gray.400}',
          floatLabelFocusColor: '{primary.color}',
          floatLabelActiveColor: '{gray.400}',
          floatLabelInvalidColor: '{form.field.invalid.placeholder.color}',
          iconColor: '{gray.400}',
          shadow: '0 0 #0000, 0 0 #0000, 0 1px 2px 0 rgba(18, 18, 23, 0.05)',
        },
        text: {
          color: '{gray.0}',
          hoverColor: '{gray.0}',
          mutedColor: '{gray.400}',
          hoverMutedColor: '{gray.300}',
        },
        content: {
          background: '{gray.900}',
          hoverBackground: '{gray.800}',
          borderColor: '{gray.700}',
          color: '{text.color}',
          hoverColor: '{text.hover.color}',
        },
        overlay: {
          select: {
            background: '{gray.700}',
            borderColor: '{gray.400}',
            color: '{text.color}',
          },
          popover: {
            background: '{gray.900}',
            borderColor: '{gray.700}',
            color: '{text.color}',
          },
          modal: {
            background: '{gray.900}',
            borderColor: '{gray.700}',
            color: '{text.color}',
          },
        },
        list: {
          option: {
            focusBackground: '{gray.800}',
            selectedBackground: '{highlight.background}',
            selectedFocusBackground: '{highlight.focus.background}',
            color: '{text.color}',
            focusColor: '{text.hover.color}',
            selectedColor: '{highlight.color}',
            selectedFocusColor: '{highlight.focus.color}',
            icon: {
              color: '{gray.500}',
              focusColor: '{gray.400}',
            },
          },
          optionGroup: {
            background: 'transparent',
            color: '{text.muted.color}',
          },
        },
        navigation: {
          item: {
            focusBackground: '{gray.800}',
            activeBackground: '{gray.800}',
            color: '{text.color}',
            focusColor: '{text.hover.color}',
            activeColor: '{text.hover.color}',
            icon: {
              color: '{gray.500}',
              focusColor: '{gray.400}',
              activeColor: '{gray.400}',
            },
          },
          submenuLabel: {
            background: 'transparent',
            color: '{text.muted.color}',
          },
          submenuIcon: {
            color: '{gray.500}',
            focusColor: '{gray.400}',
            activeColor: '{gray.400}',
          },
        },
      },
    },
  },
  components: {
    card: {
      root: {
        background: '{gray-700}',
        borderRadius: '{content.border.radius}',
        color: '{content.color}',
        shadow: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)',
      },
      title: {
        fontSize: '1.1rem',
        fontWeight: '500',
      },
      subtitle: {
        color: '{text.muted.color}',
      },
      caption: {
        gap: '0.5rem',
      },
      body: {
        padding: '1rem',
        gap: '0.5rem',
      },
    },
    progressbar: {
      root: {
        background: '{gray.900}',
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
    },
    select: {
      colorScheme: {
        light: {
          root: {
            borderRadius: '{border.radius.md}',
            color: '{red.50}',
          },
          option: {
            focusBackground: '{gray.500}',
            selectedBackground: '{gray.400}',
            selectedFocusBackground: '{gray.400}',
            color: '{red.50}',
            focusColor: '{red.50}',
            selectedColor: '{red.50}',
            selectedFocusColor: '{red.50}',
          },
        },
      },
    },
    selectbutton: {
      colorScheme: {
        light: {
          root: {
            borderRadius: '{border.radius.md}',
          },

          // option: {
          //   background: '{gray.700}',
          //   hoverBackground: '{gray.600}',
          //   selectedBackground: '{blue.500}',
          //   color: '{gray.400}',
          //   hoverColor: '{gray.200}',
          //   selectedColor: '#ffffff',
          //   borderColor: '{gray.700}',
          //   hoverBorderColor: '{gray.600}',
          //   selectedBorderColor: '{blue.500}',
          // },
        },
      },
    },
    tabs: {
      /* Used to pass tokens of the root section */
      root: {},
      /* Used to pass tokens of the tablist section */
      tablist: {
        /* Border width of tablist */
        borderWidth: '0 0 1px 0',
        /* Background of tablist */
        background: '{gray.500}',
        /* Border color of tablist */
        borderColor: '{gray.600}',
      },
      /* Used to pass tokens of the tab section */
      tab: {
        /* Background of tab */
        background: '{gray.600}',
        /* Hover background of tab */
        hoverBackground: '{grey.400}',
        /* Active background of tab */
        activeBackground: '{blue.500}',
        /* Border width of tab */
        borderWidth: '0',
        /* Border color of tab */
        borderColor: '{grey.500}',
        /* Hover border color of tab */
        hoverBorderColor: '{blue.500}',
        /* Active border color of tab */
        activeBorderColor: '{blue.500}',
        /* Color of tab */
        color: '{gray.200}',
        /* Hover color of tab */
        hoverColor: '{gray.50}',
        /* Active color of tab */
        activeColor: '{gray.100}',
        /* Padding of tab */
        padding: '0.25rem 1.125rem  0.45rem 1rem',
        /* Font weight of tab */
        fontWeight: '400',
        /* Margin of tab */
        margin: '0rem',
        /* Gap of tab */
        gap: '0rem',
        /* Focus ring of tab */
        focusRing: {
          /* Focus ring width of tab */
          width: '',
          /* Focus ring style of tab */
          style: '',
          /* Focus ring color of tab */
          color: '{red.500}',
          /* Focus ring offset of tab */
          offset: '',
          /* Focus ring shadow of tab */
          shadow: '',
        },
      },
      /* Used to pass tokens of the tabpanel section */
      tabpanel: {
        /* Background of tabpanel */
        background: '{gray.800}',
        /* Color of tabpanel */
        color: '{gray.100}',
        /* Padding of tabpanel */
        padding: '0rem',
      },
      /* Used to pass tokens of the nav button section */
      navButton: {},
      /* Used to pass tokens of the active bar section */
      activeBar: {
        /* Height of active bar */
        height: '0.35rem',
        /* Bottom of active bar */
        bottom: '0rem',
        /* Background of active bar */
        background: '{blue.500}',
      },
    },
    togglebutton: {
      colorScheme: {
        light: {
          root: {
            padding: '0.25rem',
            borderRadius: '{content.border.radius}',
            gap: '0.5rem',
            fontWeight: '500',
            disabledBackground: '{form.field.disabled.background}',
            disabledBorderColor: '{form.field.disabled.background}',
            disabledColor: '{form.field.disabled.color}',
            invalidBorderColor: '{form.field.invalid.border.color}',
            focusRing: {
              width: '{focus.ring.width}',
              style: '{focus.ring.style}',
              color: '{focus.ring.color}',
              offset: '{focus.ring.offset}',
              shadow: '{focus.ring.shadow}',
            },
            transitionDuration: '{form.field.transition.duration}',
            sm: {
              fontSize: '{form.field.sm.font.size}',
              padding: '0.25rem',
            },
            lg: {
              fontSize: '{form.field.lg.font.size}',
              padding: '0.25rem',
            },

            background: '{gray.600}',
            color: '{gray.300}',

            borderColor: '{gray-500}',
          },
          content: {
            padding: '0.25rem 0.75rem',
            borderRadius: '{content.border.radius}',
            checkedShadow: '',
            sm: {
              padding: '0.25rem 0.75rem',
            },
            lg: {
              padding: '0.25rem 0.75rem',
            },
          },
          icon: {
            color: '{gray.400}',
            checkedColor: '#ffffff',
            hoverColor: '{gray.200}',
            // checkedHoverColor: '#ffffff',
          },
        },
      },
    },
  },
});
