import { Injectable, signal, computed, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { AuthService } from './auth.service';
import { filter, take } from 'rxjs/operators';
import { toObservable } from '@angular/core/rxjs-interop';
import { BASE_URL } from './base-url.token';

interface GameSummary {
  id: number;
  status: string;
  createdAt: string;
}

@Injectable({
  providedIn: 'root'
})
export class GameStateService {
  private readonly STORAGE_KEY = 'minesweeper_active_game_id';
  private activeGameIdSignal = signal<number | null>(null);
  public activeGameId = computed(() => this.activeGameIdSignal());
  public activeGameId$ = toObservable(this.activeGameId);

  private http = inject(HttpClient);
  private authService = inject(AuthService);
  private baseUrl = inject(BASE_URL);

  constructor() {
    const savedId = localStorage.getItem(this.STORAGE_KEY);
    const initialId = savedId ? Number(savedId) : null;
    this.activeGameIdSignal.set(initialId);

    // When auth status is determined, if we're authenticated and don't have a local ID,
    // or even if we do, try to find the latest game from the server.
    toObservable(this.authService.status).pipe(
      filter(status => !status.loading && status.isAuthenticated),
      take(1) // Only do this once on load/login
    ).subscribe(() => {
      this.syncWithServer();
    });
  }

  private syncWithServer() {
    // If we already have an active game ID (e.g. from localStorage or manual selection),
    // don't overwrite it with the "latest" game.
    if (this.activeGameIdSignal() !== null) {
      console.log('Skipping sync: active game already set to', this.activeGameIdSignal());
      return;
    }

    this.http.get<GameSummary[]>(this.baseUrl + 'user/games').subscribe(games => {
      if (games && games.length > 0) {
        const activeGames = games.filter(g => g.status === 'InProgress');
        if (activeGames.length > 0) {
          // Sort by CreatedAt descending to get the truly latest game
          const latestGame = activeGames.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())[0];
          
          // Only update if it's different from what we have
          if (this.activeGameIdSignal() !== latestGame.id) {
            console.log('Syncing with server: found active game', latestGame.id);
            this.setActiveGame(latestGame.id);
          }
        }
      }
    });
  }

  setActiveGame(id: number | null) {
    if (id === null) {
      localStorage.removeItem(this.STORAGE_KEY);
    } else {
      localStorage.setItem(this.STORAGE_KEY, id.toString());
    }
    this.activeGameIdSignal.set(id);
  }

  getActiveGameId(): number | null {
    return this.activeGameIdSignal();
  }
}

