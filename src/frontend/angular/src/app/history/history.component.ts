import { Component, OnInit } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { AuthService } from '../auth.service';
import { GameStateService } from '../game-state.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-history',
  templateUrl: './history.component.html'
})
export class HistoryComponent implements OnInit {
  public games: any[] = [];
  public loading = true;

  constructor(
    private http: HttpClient, 
    private authService: AuthService,
    private gameStateService: GameStateService,
    private router: Router) { }

  ngOnInit(): void {
    this.http.get<any[]>('/user/games').subscribe(
      result => {
        this.games = result;
        this.loading = false;
      },
      error => {
        console.error('Could not fetch game history', error);
        this.loading = false;
      }
    );
  }

  resumeGame(id: number) {
    this.gameStateService.setActiveGame(id);
    this.router.navigate(['/minesweeper']);
  }
}

