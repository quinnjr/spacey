import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    loadComponent: () => import('./pages/home/home').then(m => m.HomeComponent)
  },
  {
    path: 'crates',
    loadComponent: () => import('./pages/crates/crates').then(m => m.CratesComponent)
  },
  {
    path: 'benchmarks',
    loadComponent: () => import('./pages/benchmarks/benchmarks').then(m => m.BenchmarksComponent)
  }
];
