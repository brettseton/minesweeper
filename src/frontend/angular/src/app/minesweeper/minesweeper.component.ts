import { Component, OnInit } from '@angular/core';
import { AuthService } from '../auth.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-minesweeper',
  templateUrl: './minesweeper.component.html'
})
export class MinesweeperComponent implements OnInit {

  constructor(private authService: AuthService, private router: Router) {
  }

  ngOnInit() {
    // We no longer redirect to login. Guests can play.
  }
}


