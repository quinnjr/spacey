import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'welcome',
    pathMatch: 'full'
  },
  {
    path: 'welcome',
    loadComponent: () => import('./pages/welcome/welcome.component').then(m => m.WelcomeComponent),
    title: 'Welcome - Spacey Browser'
  },
  {
    path: 'settings',
    loadComponent: () => import('./pages/settings/settings.component').then(m => m.SettingsComponent),
    title: 'Settings - Spacey Browser'
  },
  {
    path: 'bugreport',
    loadComponent: () => import('./pages/bugreport/bugreport.component').then(m => m.BugreportComponent),
    title: 'Report a Bug - Spacey Browser'
  },
  {
    path: 'bugreport-thanks',
    loadComponent: () => import('./pages/bugreport-thanks/bugreport-thanks.component').then(m => m.BugreportThanksComponent),
    title: 'Thank You - Spacey Browser'
  },
  {
    path: '**',
    redirectTo: 'welcome'
  }
];
