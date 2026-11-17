import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { App } from './app/app';

bootstrapApplication(App, appConfig)
  .then(() => document.getElementById('startup-status')?.remove())
  .catch((error: unknown) => {
    console.error(error);
    window.dispatchEvent(new CustomEvent('hoi4reader:startup-error', { detail: String(error) }));
  });
