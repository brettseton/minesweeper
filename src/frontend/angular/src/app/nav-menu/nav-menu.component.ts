import { Component, inject } from '@angular/core';
import { GameStateService } from '../game-state.service';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { NgClass } from '@angular/common';
import { LoginMenuComponent } from './login-menu/login-menu.component';

@Component({
  selector: 'app-nav-menu',
  templateUrl: './nav-menu.component.html',
  styleUrls: ['./nav-menu.component.css'],
  standalone: true,
  imports: [NgClass, RouterLink, RouterLinkActive, LoginMenuComponent]
})
export class NavMenuComponent {
  isExpanded = false;

  private gameStateService = inject(GameStateService);

  collapse() {
    this.isExpanded = false;
  }

  toggle() {
    this.isExpanded = !this.isExpanded;
  }
}
