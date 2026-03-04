import type { Plugin } from 'vue';
import * as PrimeVue from 'primevue';

const componentNames = [
  'Button', 'Card', 'Checkbox', 'Chip', 'ContextMenu', 'Dialog',
  'Divider', 'InputGroup', 'InputGroupAddon', 'InputNumber', 'InputText',
  'Message', 'Popover', 'ProgressBar', 'Select', 'SelectButton', 'Slider',
  'Tab', 'TabList', 'TabPanel', 'TabPanels', 'Tabs', 'Tag', 'Textarea',
  'ToggleSwitch', 'Toolbar',
] as const;

export const PrimeVueComponents: Plugin = {
  install(app) {
    for (const name of componentNames) {
      app.component(name, PrimeVue[name]);
    }
  },
};