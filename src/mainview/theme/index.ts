import type { Preset } from '@primeuix/themes/types';

import accordion from './accordion';
import autocomplete from './autocomplete';
import avatar from './avatar';
import badge from './badge';
import blockui from './blockui';
import breadcrumb from './breadcrumb';
import button from './button';
import card from './card';
import carousel from './carousel';
import cascadeselect from './cascadeselect';
import checkbox from './checkbox';
import chip from './chip';
import colorpicker from './colorpicker';
import confirmdialog from './confirmdialog';
import confirmpopup from './confirmpopup';
import contextmenu from './contextmenu';
import css from './css';
import datatable from './datatable';
import dataview from './dataview';
import datepicker from './datepicker';
import dialog from './dialog';
import divider from './divider';
import dock from './dock';
import drawer from './drawer';
import editor from './editor';
import fieldset from './fieldset';
import fileupload from './fileupload';
import floatlabel from './floatlabel';
import galleria from './galleria';
import iconfield from './iconfield';
import iftalabel from './iftalabel';
import image from './image';
import imagecompare from './imagecompare';
import inlinemessage from './inlinemessage';
import inplace from './inplace';
import inputchips from './inputchips';
import inputgroup from './inputgroup';
import inputnumber from './inputnumber';
import inputotp from './inputotp';
import inputtext from './inputtext';
import knob from './knob';
import listbox from './listbox';
import megamenu from './megamenu';
import menu from './menu';
import menubar from './menubar';
import message from './message';
import metergroup from './metergroup';
import multiselect from './multiselect';
import orderlist from './orderlist';
import organizationchart from './organizationchart';
import overlaybadge from './overlaybadge';
import paginator from './paginator';
import panel from './panel';
import panelmenu from './panelmenu';
import password from './password';
import picklist from './picklist';
import popover from './popover';
import { primitive } from './primitives';
import progressbar from './progressbar';
import progressspinner from './progressspinner';
import radiobutton from './radiobutton';
import rating from './rating';
import ripple from './ripple';
import scrollpanel from './scrollpanel';
import select from './select';
import selectbutton from './selectbutton';
import skeleton from './skeleton';
import slider from './slider';
import speeddial from './speeddial';
import splitbutton from './splitbutton';
import splitter from './splitter';
import stepper from './stepper';
import steps from './steps';
import tabmenu from './tabmenu';
import tabs from './tabs';
import tabview from './tabview';
import tag from './tag';
import terminal from './terminal';
import textarea from './textarea';
import tieredmenu from './tieredmenu';
import timeline from './timeline';
import toast from './toast';
import togglebutton from './togglebutton';
import toggleswitch from './toggleswitch';
import toolbar from './toolbar';
import tooltip from './tooltip';
import tree from './tree';
import treeselect from './treeselect';
import treetable from './treetable';
import virtualscroller from './virtualscroller';
import { GlassBaseDesignTokens, GlassBaseTokenSections } from './types';

export const semantic: GlassBaseTokenSections.Semantic = {
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
    paddingX: '0.55rem',
    paddingY: '0.3rem',
    sm: {
      fontSize: '1rem',
      paddingX: '0.625rem',
      paddingY: '0.15rem',
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
      padding: '0.25rem 1.0rem',
      borderRadius: '{border.radius.sm}',
    },
    optionGroup: {
      padding: '0.5rem 0.15rem',
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
        0: '#ffffff',
        50: '{slate.50}',
        100: '{slate.100}',
        200: '{slate.200}',
        300: '{slate.300}',
        400: '{slate.400}',
        500: '{slate.500}',
        600: '{slate.600}',
        700: '{slate.700}',
        800: '{slate.800}',
        900: '{slate.900}',
        950: '{slate.950}',
      },
      primary: {
        color: '{primary.500}',
        contrastColor: '#ffffff',
        hoverColor: '{primary.600}',
        activeColor: '{primary.700}',
      },
      highlight: {
        background: '{primary.50}',
        focusBackground: '{primary.100}',
        color: '{primary.700}',
        focusColor: '{primary.800}',
      },
      mask: {
        background: 'rgba(0,0,0,0.4)',
        color: '{surface.200}',
      },
      formField: {
        background: '{surface.0}',
        disabledBackground: '{surface.200}',
        filledBackground: '{surface.50}',
        filledHoverBackground: '{surface.50}',
        filledFocusBackground: '{surface.50}',
        borderColor: '{surface.300}',
        hoverBorderColor: '{surface.400}',
        focusBorderColor: '{primary.color}',
        invalidBorderColor: '{red.400}',
        color: '{surface.700}',
        disabledColor: '{surface.500}',
        placeholderColor: '{surface.500}',
        invalidPlaceholderColor: '{red.600}',
        floatLabelColor: '{surface.500}',
        floatLabelFocusColor: '{primary.600}',
        floatLabelActiveColor: '{surface.500}',
        floatLabelInvalidColor: '{form.field.invalid.placeholder.color}',
        iconColor: '{surface.400}',
        shadow: '0 0 #0000, 0 0 #0000, 0 1px 2px 0 rgba(18, 18, 23, 0.05)',
      },
      text: {
        color: '{surface.700}',
        hoverColor: '{surface.800}',
        mutedColor: '{surface.500}',
        hoverMutedColor: '{surface.600}',
      },
      content: {
        background: '{surface.0}',
        hoverBackground: '{surface.100}',
        borderColor: '{surface.200}',
        color: '{text.color}',
        hoverColor: '{text.hover.color}',
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
          background: '{surface.0}',
          borderColor: '{surface.200}',
          color: '{text.color}',
        },
      },
      list: {
        option: {
          focusBackground: '{surface.100}',
          selectedBackground: '{highlight.background}',
          selectedFocusBackground: '{highlight.focus.background}',
          color: '{text.color}',
          focusColor: '{text.hover.color}',
          selectedColor: '{highlight.color}',
          selectedFocusColor: '{highlight.focus.color}',
          icon: {
            color: '{surface.400}',
            focusColor: '{surface.500}',
          },
        },
        optionGroup: {
          background: 'transparent',
          color: '{text.muted.color}',
        },
      },
      navigation: {
        item: {
          focusBackground: '{surface.100}',
          activeBackground: '{surface.100}',
          color: '{text.color}',
          focusColor: '{text.hover.color}',
          activeColor: '{text.hover.color}',
          icon: {
            color: '{surface.400}',
            focusColor: '{surface.500}',
            activeColor: '{surface.500}',
          },
        },
        submenuLabel: {
          background: 'transparent',
          color: '{text.muted.color}',
        },
        submenuIcon: {
          color: '{surface.400}',
          focusColor: '{surface.500}',
          activeColor: '{surface.500}',
        },
      },
    },
    dark: {
      surface: {
        0: '#ffffff',
        50: '{zinc.50}',
        100: '{zinc.100}',
        200: '{zinc.200}',
        300: '{zinc.300}',
        400: '{zinc.400}',
        500: '{zinc.500}',
        600: '{zinc.600}',
        700: '{zinc.700}',
        800: '{zinc.800}',
        900: '{zinc.900}',
        950: '{zinc.950}',
      },
      primary: {
        color: '{primary.400}',
        contrastColor: '{surface.900}',
        hoverColor: '{primary.300}',
        activeColor: '{primary.200}',
      },
      highlight: {
        background: 'color-mix(in srgb, {primary.400}, transparent 84%)',
        focusBackground: 'color-mix(in srgb, {primary.400}, transparent 76%)',
        color: 'rgba(255,255,255,.87)',
        focusColor: 'rgba(255,255,255,.87)',
      },
      mask: {
        background: 'rgba(0,0,0,0.6)',
        color: '{surface.200}',
      },
      formField: {
        background: '{surface.950}',
        disabledBackground: '{surface.700}',
        filledBackground: '{surface.800}',
        filledHoverBackground: '{surface.800}',
        filledFocusBackground: '{surface.800}',
        borderColor: '{surface.600}',
        hoverBorderColor: '{surface.500}',
        focusBorderColor: '{primary.color}',
        invalidBorderColor: '{red.300}',
        color: '{surface.0}',
        disabledColor: '{surface.400}',
        placeholderColor: '{surface.400}',
        invalidPlaceholderColor: '{red.400}',
        floatLabelColor: '{surface.400}',
        floatLabelFocusColor: '{primary.color}',
        floatLabelActiveColor: '{surface.400}',
        floatLabelInvalidColor: '{form.field.invalid.placeholder.color}',
        iconColor: '{surface.400}',
        shadow: '0 0 #0000, 0 0 #0000, 0 1px 2px 0 rgba(18, 18, 23, 0.05)',
      },
      text: {
        color: '{surface.0}',
        hoverColor: '{surface.0}',
        mutedColor: '{surface.400}',
        hoverMutedColor: '{surface.300}',
      },
      content: {
        background: '{surface.900}',
        hoverBackground: '{surface.800}',
        borderColor: '{surface.700}',
        color: '{text.color}',
        hoverColor: '{text.hover.color}',
      },
      overlay: {
        select: {
          background: '{surface.900}',
          borderColor: '{surface.700}',
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
      list: {
        option: {
          focusBackground: '{surface.800}',
          selectedBackground: '{highlight.background}',
          selectedFocusBackground: '{highlight.focus.background}',
          color: '{text.color}',
          focusColor: '{text.hover.color}',
          selectedColor: '{highlight.color}',
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
      navigation: {
        item: {
          focusBackground: '{surface.800}',
          activeBackground: '{surface.800}',
          color: '{text.color}',
          focusColor: '{text.hover.color}',
          activeColor: '{text.hover.color}',
          icon: {
            color: '{surface.500}',
            focusColor: '{surface.400}',
            activeColor: '{surface.400}',
          },
        },
        submenuLabel: {
          background: 'transparent',
          color: '{text.muted.color}',
        },
        submenuIcon: {
          color: '{surface.500}',
          focusColor: '{surface.400}',
          activeColor: '{surface.400}',
        },
      },
    },
  },
};

export default {
  primitive,
  semantic,
  components: {
    accordion,
    autocomplete,
    avatar,
    badge,
    blockui,
    breadcrumb,
    button,
    card,
    carousel,
    cascadeselect,
    checkbox,
    chip,
    colorpicker,
    confirmdialog,
    confirmpopup,
    contextmenu,
    datatable,
    dataview,
    datepicker,
    dialog,
    divider,
    dock,
    drawer,
    editor,
    fieldset,
    fileupload,
    floatlabel,
    galleria,
    iconfield,
    iftalabel,
    image,
    imagecompare,
    inlinemessage,
    inplace,
    inputchips,
    inputgroup,
    inputnumber,
    inputotp,
    inputtext,
    knob,
    listbox,
    megamenu,
    menu,
    menubar,
    message,
    metergroup,
    multiselect,
    orderlist,
    organizationchart,
    overlaybadge,
    paginator,
    panel,
    panelmenu,
    password,
    picklist,
    popover,
    progressbar,
    progressspinner,
    radiobutton,
    rating,
    ripple,
    scrollpanel,
    select,
    selectbutton,
    skeleton,
    slider,
    speeddial,
    splitbutton,
    splitter,
    stepper,
    steps,
    tabmenu,
    tabs,
    tabview,
    tag,
    terminal,
    textarea,
    tieredmenu,
    timeline,
    toast,
    togglebutton,
    toggleswitch,
    toolbar,
    tooltip,
    tree,
    treeselect,
    treetable,
    virtualscroller,
  },
  css,
} satisfies Preset<GlassBaseDesignTokens>;
