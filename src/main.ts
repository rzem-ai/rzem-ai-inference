import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';
import ConfirmationService from 'primevue/confirmationservice';
import Tooltip from 'primevue/tooltip';
import MasonryWall from '@/components/shared/wall/masonry-wall';
import router from './router';
import App from './App.vue';

/* import the fontawesome core */
import { library } from '@fortawesome/fontawesome-svg-core';

/* import font awesome icon component */
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome';

/* import specific icons */
import { fad } from '@fortawesome/pro-duotone-svg-icons';
import { fal } from '@fortawesome/pro-light-svg-icons';
import { far } from '@fortawesome/pro-regular-svg-icons';
import { fas } from '@fortawesome/pro-solid-svg-icons';
import { fat } from '@fortawesome/pro-thin-svg-icons';

/* add icons to the library */
library.add(fad, fal, far, fas, fat);

import { AuraPlus } from './theme';
import './style.css';

const app = createApp(App);
const pinia = createPinia();

app.component('fa', FontAwesomeIcon);
app.directive('tooltip', Tooltip);
app.use(pinia);
app.use(router);
app.use(ToastService);
app.use(ConfirmationService);
app.use(MasonryWall);
app.use(PrimeVue, {
  theme: {
    preset: AuraPlus,
    options: {
      darkModeSelector: '.dark',
      cssLayer: {
        name: 'ui',
        order: 'theme, base, ui',
      },
    },
  },
});

app.mount('#app');
