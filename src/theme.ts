import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

/**
 * shadcn/ui inspired dark theme using Tailwind Gray palette
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
      offset: '2px',
      shadow: 'none',
    },
    disabledOpacity: '0.6',
    iconSize: '1rem',
    anchorGutter: '2px',

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
      light: {
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
        secondary: {
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
      },
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
          activeColor: '{red.500}',
          color: '{blue.500}',
          contrastColor: '{surface.100}',
          hoverColor: '{blue.600}',
        },
        secondary: {
          activeColor: '{green.500}',
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
        text: {
          color: '{surface.0}',
          hoverColor: '{surface.0}',
          hoverMutedColor: '{surface.300}',
          mutedColor: '{surface.400}',
        },
      },
    },
  },
  components: {
    card: {
      root: {
        background: '{surface-700}',
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
    inputnumber: {
      root: {
        transitionDuration: '{transition.duration}',
      },
      button: {
        width: '2.5rem',
        borderRadius: '{form.field.border.radius}',
        verticalPadding: '{form.field.padding.y}',
      },
      colorScheme: {
        light: {
          button: {
            background: 'transparent',
            hoverBackground: '{surface.100}',
            activeBackground: '{surface.200}',
            borderColor: '{form.field.border.color}',
            hoverBorderColor: '{form.field.border.color}',
            activeBorderColor: '{form.field.border.color}',
            color: '{surface.400}',
            hoverColor: '{surface.500}',
            activeColor: '{surface.600}',
          },
        },
        dark: {
          button: {
            background: 'transparent',
            hoverBackground: '{surface.800}',
            activeBackground: '{surface.700}',
            borderColor: '{form.field.border.color}',
            hoverBorderColor: '{form.field.border.color}',
            activeBorderColor: '{form.field.border.color}',
            color: '{surface.400}',
            hoverColor: '{surface.300}',
            activeColor: '{surface.200}',
          },
        },
      },
    },
    inputtext: {
      root: {
        paddingX: '{form.field.padding.x}',
        paddingY: '{form.field.padding.y}',
        borderRadius: '{form.field.border.radius}',
        focusRing: {
          width: '{form.field.focus.ring.width}',
          style: '{form.field.focus.ring.style}',
          offset: '{form.field.focus.ring.offset}',
          shadow: '{form.field.focus.ring.shadow}',
        },
        transitionDuration: '{form.field.transition.duration}',
        sm: {
          fontSize: '{form.field.sm.font.size}',
          paddingX: '{form.field.sm.padding.x}',
          paddingY: '{form.field.sm.padding.y}',
        },
        lg: {
          fontSize: '{form.field.lg.font.size}',
          paddingX: '{form.field.lg.padding.x}',
          paddingY: '{form.field.lg.padding.y}',
        },
      },
      colorScheme: {
        light: {
          root: {
            background: '{red.500}',
            disabledBackground: '{form.field.disabled.background}',
            filledBackground: '{form.field.filled.background}',
            filledHoverBackground: '{form.field.filled.hover.background}',
            filledFocusBackground: '{form.field.filled.focus.background}',
            borderColor: '{form.field.border.color}',
            hoverBorderColor: '{form.field.hover.border.color}',
            focusBorderColor: '{form.field.focus.border.color}',
            invalidBorderColor: '{form.field.invalid.border.color}',
            color: '{form.field.color}',
            disabledColor: '{form.field.disabled.color}',
            placeholderColor: '{form.field.placeholder.color}',
            invalidPlaceholderColor: '{form.field.invalid.placeholder.color}',
            shadow: '{form.field.shadow}',
            paddingX: '{form.field.padding.x}',
            paddingY: '{form.field.padding.y}',
            borderRadius: '{form.field.border.radius}',
            focusRing: {
              width: '{form.field.focus.ring.width}',
              style: '{form.field.focus.ring.style}',
              color: '{form.field.focus.ring.color}',
              offset: '{form.field.focus.ring.offset}',
              shadow: '{form.field.focus.ring.shadow}',
            },
            transitionDuration: '{form.field.transition.duration}',
            sm: {
              fontSize: '{form.field.sm.font.size}',
              paddingX: '{form.field.sm.padding.x}',
              paddingY: '{form.field.sm.padding.y}',
            },
            lg: {
              fontSize: '{form.field.lg.font.size}',
              paddingX: '{form.field.lg.padding.x}',
              paddingY: '{form.field.lg.padding.y}',
            },
          },
        },
        dark: {
          root: {
            background: '{surface.900}',
            disabledBackground: '{form.field.disabled.background}',
            filledBackground: '{form.field.filled.background}',
            filledHoverBackground: '{form.field.filled.hover.background}',
            filledFocusBackground: '{form.field.filled.focus.background}',
            borderColor: '{surface.700}',
            hoverBorderColor: '{form.field.hover.border.color}',
            focusBorderColor: '{form.field.focus.border.color}',
            invalidBorderColor: '{form.field.invalid.border.color}',
            color: '{form.field.color}',
            disabledColor: '{form.field.disabled.color}',
            placeholderColor: '{form.field.placeholder.color}',
            invalidPlaceholderColor: '{form.field.invalid.placeholder.color}',
            shadow: '{form.field.shadow}',
            paddingX: '{form.field.padding.x}',
            paddingY: '{form.field.padding.y}',
            borderRadius: '{form.field.border.radius}',
            focusRing: {
              width: '{form.field.focus.ring.width}',
              style: '{form.field.focus.ring.style}',
              color: '{form.field.focus.ring.color}',
              offset: '{form.field.focus.ring.offset}',
              shadow: '{form.field.focus.ring.shadow}',
            },
            transitionDuration: '{form.field.transition.duration}',
            sm: {
              fontSize: '{form.field.sm.font.size}',
              paddingX: '{form.field.sm.padding.x}',
              paddingY: '{form.field.sm.padding.y}',
            },
            lg: {
              fontSize: '{form.field.lg.font.size}',
              paddingX: '{form.field.lg.padding.x}',
              paddingY: '{form.field.lg.padding.y}',
            },
          },
        },
      },
    },
    panel: {
      root: {
        borderRadius: '0rem',
      },
      content: {
        padding: '0rem',
      },
      footer: {
        padding: '0rem',
      },
      header: {
        padding: '0rem',
        borderWidth: '0rem',
        borderRadius: '0rem',
      },
      title: {
        fontWeight: '600',
      },
      toggleableHeader: {
        padding: '0rem',
      },
      colorScheme: {
        light: {
          root: {
            background: '{content.background}',
            borderColor: '{content.border.color}',
            color: '{content.color}',
          },
          content: {},
          footer: {},
          header: {
            background: 'transparent',
            color: '{text.color}',
            borderColor: '{content.border.color}',
          },
          title: {},
          toggleableHeader: {},
        },
        dark: {
          root: {
            background: 'transparent',
            borderColor: 'transparent',
            color: '{content.color}',
          },
          content: {},
          footer: {},
          header: {
            background: 'transparent',
            color: '{text.color}',
            borderColor: '{content.border.color}',
          },
          title: {},
          toggleableHeader: {},
        },
      },
    },
    progressbar: {
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
    },
    select: {
      root: {
        borderRadius: '{border.radius.md}',
      },
      option: {},
      colorScheme: {
        light: {
          root: {
            color: '{red.50}',
          },
          option: {
            focusBackground: '{surface.500}',
            selectedBackground: '{surface.400}',
            selectedFocusBackground: '{surface.400}',
            color: '{red.50}',
            focusColor: '{red.50}',
            selectedColor: '{red.50}',
            selectedFocusColor: '{red.50}',
          },
        },
        dark: {
          root: {
            color: '{red.50}',
          },
          option: {
            focusBackground: '{surface.500}',
            selectedBackground: '{surface.400}',
            selectedFocusBackground: '{surface.400}',
            color: '{red.50}',
            focusColor: '{red.50}',
            selectedColor: '{red.50}',
            selectedFocusColor: '{red.50}',
          },
        },
      },
    },
    selectbutton: {
      root: {
        borderRadius: '{border.radius.md}',
      },
      colorScheme: {
        light: {
          root: {},
        },
        dark: {
          root: {},
        },
      },
    },
    slider: {
      root: {
        transitionDuration: '{transition.duration}',
      },
      handle: {
        width: '20px',
        height: '20px',
        borderRadius: '50%',
        content: {
          borderRadius: '50%',
          hoverBackground: '{content.background}',
        },
        focusRing: {
          width: '{focus.ring.width}',
          style: '{focus.ring.style}',
          offset: '{focus.ring.offset}',
        },
      },
      range: {},
      track: {
        borderRadius: '{content.border.radius}',
        size: '3px',
      },
      colorScheme: {
        light: {
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
    },
    tabs: {
      /* Used to pass tokens of the root section */
      root: {},
      /* Used to pass tokens of the tablist section */
      tablist: {
        /* Border width of tablist */
        borderWidth: '0 0 1px 0',
        /* Background of tablist */
        background: '{surface.500}',
        /* Border color of tablist */
        borderColor: '{surface.600}',
      },
      /* Used to pass tokens of the tab section */
      tab: {
        /* Background of tab */
        background: '{surface.600}',
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
        color: '{surface.200}',
        /* Hover color of tab */
        hoverColor: '{surface.50}',
        /* Active color of tab */
        activeColor: '{surface.100}',
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
        background: '{surface.800}',
        /* Color of tabpanel */
        color: '{surface.100}',
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

            background: '{surface.600}',
            color: '{surface.300}',

            borderColor: '{surface-500}',
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
            color: '{surface.400}',
            checkedColor: '#ffffff',
            hoverColor: '{surface.200}',
            // checkedHoverColor: '#ffffff',
          },
        },
        dark: {
          root: {
            background: '{surface.900}',
            borderColor: '{surface.900}',
            borderRadius: '{content.border.radius}',
            checkedBackground: '{surface.900}',
            checkedBorderColor: '{surface.900}',
            checkedColor: '{surface.0}',
            color: '{surface.100}',
            disabledBackground: '{form.field.disabled.background}',
            disabledBorderColor: '{form.field.disabled.background}',
            disabledColor: '{form.field.disabled.color}',
            fontWeight: '500',
            gap: '0.5rem',
            hoverBackground: '{surface.950}',
            hoverColor: '{surface.100}',
            invalidBorderColor: '{form.field.invalid.border.color}',
            padding: '0.25rem',
            transitionDuration: '{form.field.transition.duration}',
            focusRing: {
              width: '{focus.ring.width}',
              style: '{focus.ring.style}',
              color: '{focus.ring.color}',
              offset: '{focus.ring.offset}',
              shadow: '{focus.ring.shadow}',
            },
            sm: {
              fontSize: '{form.field.sm.font.size}',
              padding: '0.25rem',
            },
            lg: {
              fontSize: '{form.field.lg.font.size}',
              padding: '0.25rem',
            },
          },
          content: {
            padding: '0.25rem 0.75rem',
            borderRadius: '{content.border.radius}',
            checkedBackground: '{surface.500}',
            sm: {
              padding: '0.25rem 0.75rem',
            },
            lg: {
              padding: '0.25rem 0.75rem',
            },
          },
          icon: {
            color: '{surface.400}',
            hoverColor: '{surface.200}',
            checkedColor: '{surface.0}',
          },
        },
      },
    },
  },
});
