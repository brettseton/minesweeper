import { Injectable, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, tap } from 'rxjs';
import { MinesweeperGame, Point } from './models';
import { BASE_URL } from '../base-url.token';
import { metrics } from '@opentelemetry/api';

@Injectable({
  providedIn: 'root'
})
export class MinesweeperApiService {
  private http = inject(HttpClient);
  private baseUrl = inject(BASE_URL);
  private meter = metrics.getMeter('minesweeper-api');
  private moveCounter = this.meter.createCounter('minesweeper.moves', {
    description: 'Number of moves made',
  });
  private gameCounter = this.meter.createCounter('minesweeper.games_started', {
    description: 'Number of new games started',
  });

  getGame(id: number): Observable<MinesweeperGame> {
    return this.http.get<MinesweeperGame>(`${this.baseUrl}game/${id}`);
  }

  getNewGame(cols: number = 10, rows: number = 10, mines: number = 10): Observable<MinesweeperGame> {
    return this.http.get<MinesweeperGame>(`${this.baseUrl}game/new/${cols}/${rows}/${mines}`).pipe(
      tap(() => this.gameCounter.add(1))
    );
  }

  makeMove(point: Point): Observable<MinesweeperGame> {
    return this.http.post<MinesweeperGame>(`${this.baseUrl}game`, point).pipe(
      tap(() => this.moveCounter.add(1))
    );
  }

  toggleFlag(point: Point): Observable<MinesweeperGame> {
    return this.http.post<MinesweeperGame>(`${this.baseUrl}game/flag`, point);
  }
}
