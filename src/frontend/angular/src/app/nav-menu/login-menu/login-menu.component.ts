import { Component, signal, inject } from '@angular/core';
import { AuthService } from '../../auth.service';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-login-menu',
  templateUrl: './login-menu.component.html',
  styleUrls: ['./login-menu.component.css'],
  standalone: true,
  imports: [RouterLink]
})
export class LoginMenuComponent {
  isUserMenuExpanded = signal(false);

  public authService = inject(AuthService);

  toggleUserMenu() {
    this.isUserMenuExpanded.update(v => !v);
  }

  closeUserMenu() {
    this.isUserMenuExpanded.set(false);
  }

  logout() {
    this.closeUserMenu();
    this.authService.logout();
  }
}

