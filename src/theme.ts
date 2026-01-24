import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

import { components } from '@/components/theme/components';
import { primitive } from '@/components/theme/primitive';
import { semantic } from '@/components/theme/semantic';

export const AuraPlus = definePreset(Aura, {
  primitive,
  semantic,
  components,
});
