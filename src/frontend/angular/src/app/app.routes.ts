import { Routes } from '@angular/router';
import { HomeComponent } from './home/home.component';
import { LoginComponent } from './login/login.component';
import { HistoryComponent } from './history/history.component';
import { StatsComponent } from './stats/stats.component';
import { MinesweeperBoardComponent } from './minesweeper-board/minesweeper-board.component';
import { PrivacyComponent } from './privacy/privacy.component';
import { AuthGuard } from './auth.guard';

export const routes: Routes = [
  { path: '', component: HomeComponent, pathMatch: 'full' },
  { path: 'login', component: LoginComponent },
  { path: 'history', component: HistoryComponent, canActivate: [AuthGuard] },
  { path: 'stats', component: StatsComponent, canActivate: [AuthGuard] },
  { path: 'minesweeper', component: MinesweeperBoardComponent },
  { path: 'minesweeper/:id', component: MinesweeperBoardComponent },
  { path: 'privacy', component: PrivacyComponent },
  
  // External redirects
  { 
    path: 'account/google-login', 
    canActivate: [() => { window.location.href = '/account/google-login'; return false; }], 
    component: HomeComponent 
  },
  { 
    path: 'account/google-logout', 
    canActivate: [() => { window.location.href = '/account/google-logout'; return false; }], 
    component: HomeComponent 
  },
  { 
    path: 'signin-google', 
    canActivate: [() => { window.location.href = '/signin-google'; return false; }], 
    component: HomeComponent 
  },
];
