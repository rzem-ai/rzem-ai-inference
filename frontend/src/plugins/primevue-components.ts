import type { Plugin } from 'vue';
import {
  Button,
  Card,
  Checkbox,
  Chip,
  ContextMenu,
  Dialog,
  Divider,
  InputGroup,
  InputGroupAddon,
  InputNumber,
  InputText,
  Message,
  Popover,
  ProgressBar,
  Select,
  SelectButton,
  Slider,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Tag,
  Textarea,
  Toolbar,
} from 'primevue';

const components: Record<string, any> = {
  Button,
  Card,
  Checkbox,
  Chip,
  ContextMenu,
  Dialog,
  Divider,
  InputGroup,
  InputGroupAddon,
  InputNumber,
  InputText,
  Message,
  Popover,
  ProgressBar,
  Select,
  SelectButton,
  Slider,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Tag,
  Textarea,
  Toolbar,
};

export const PrimeVueComponents: Plugin = {
  install(app) {
    for (const [name, component] of Object.entries(components)) {
      app.component(name, component);
    }
  },
};
