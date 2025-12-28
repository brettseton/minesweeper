import { Injectable, Inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { BehaviorSubject } from 'rxjs';
import { AuthService } from './auth.service';
import { filter, take } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class GameStateService {
  private readonly STORAGE_KEY = 'minesweeper_active_game_id';
  private activeGameIdSubject: BehaviorSubject<number | null>;
  public activeGameId$;

  constructor(
    private http: HttpClient,
    private authService: AuthService,
    @Inject('BASE_URL') private baseUrl: string
  ) {
    const savedId = localStorage.getItem(this.STORAGE_KEY);
    const initialId = savedId ? Number(savedId) : null;
    this.activeGameIdSubject = new BehaviorSubject<number | null>(initialId);
    this.activeGameId$ = this.activeGameIdSubject.asObservable();

    // When auth status is determined, if we're authenticated and don't have a local ID,
    // or even if we do, try to find the latest game from the server.
    this.authService.status$.pipe(
      filter(status => !status.loading && status.isAuthenticated),
      take(1) // Only do this once on load/login
    ).subscribe(() => {
      this.syncWithServer();
    });
  }

  private syncWithServer() {
    // If we already have an active game ID (e.g. from localStorage or manual selection),
    // don't overwrite it with the "latest" game.
    if (this.activeGameIdSubject.value !== null) {
      console.log('Skipping sync: active game already set to', this.activeGameIdSubject.value);
      return;
    }

    this.http.get<any[]>(this.baseUrl + 'user/games').subscribe(games => {
      if (games && games.length > 0) {
        const activeGames = games.filter(g => g.status === 'InProgress');
        if (activeGames.length > 0) {
          // Sort by CreatedAt descending to get the truly latest game
          const latestGame = activeGames.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())[0];
          
          // Only update if it's different from what we have
          if (this.activeGameIdSubject.value !== latestGame.id) {
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
    this.activeGameIdSubject.next(id);
  }

  getActiveGameId(): number | null {
    return this.activeGameIdSubject.value;
  }
}

