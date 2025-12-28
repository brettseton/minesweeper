import { Component } from '@angular/core';
import { AuthService } from '../../auth.service';

@Component({
  selector: 'app-login-menu',
  templateUrl: './login-menu.component.html',
  styleUrls: ['./login-menu.component.css']
})
export class LoginMenuComponent {
  isUserMenuExpanded = false;

  constructor(public authService: AuthService) {}

  toggleUserMenu() {
    this.isUserMenuExpanded = !this.isUserMenuExpanded;
  }

  closeUserMenu() {
    this.isUserMenuExpanded = false;
  }

  logout() {
    this.closeUserMenu();
    this.authService.logout();
  }
}

