import { Component, OnInit, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { GameStateService } from '../game-state.service';
import { Router } from '@angular/router';
import { NgClass } from '@angular/common';

interface GameHistoryItem {
  id: number;
  status: string;
  mineCount: number;
}

@Component({
  selector: 'app-history',
  templateUrl: './history.component.html',
  standalone: true,
  imports: [NgClass]
})
export class HistoryComponent implements OnInit {
  public games: GameHistoryItem[] = [];
  public loading = true;

  private http = inject(HttpClient);
  private gameStateService = inject(GameStateService);
  private router = inject(Router);

  ngOnInit(): void {
    this.http.get<GameHistoryItem[]>('/user/games').subscribe({
      next: (result) => {
        this.games = result;
        this.loading = false;
      },
      error: (error) => {
        console.error('Could not fetch game history', error);
        this.loading = false;
      }
    });
  }

  resumeGame(id: number) {
    this.gameStateService.setActiveGame(id);
    this.router.navigate(['/minesweeper']);
  }
}

