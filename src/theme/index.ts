import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

import { components } from './components';
import { primitive } from './primitive';
import { semantic } from './semantic';

export const AuraPlus = definePreset(Aura, {
  primitive,
  semantic,
  components,
});
