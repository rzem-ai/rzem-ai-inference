import type { TabsDesignTokens } from '@primeuix/themes/types/tabs';

export const tabs: TabsDesignTokens = {
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
};
