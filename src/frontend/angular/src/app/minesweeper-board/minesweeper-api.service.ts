import { Injectable, Inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { MinesweeperGame, Point } from './models';

@Injectable({
  providedIn: 'root'
})
export class MinesweeperApiService {
  constructor(
    private http: HttpClient,
    @Inject('BASE_URL') private baseUrl: string
  ) {}

  getGame(id: number): Observable<MinesweeperGame> {
    return this.http.get<MinesweeperGame>(`${this.baseUrl}game/${id}`);
  }

  getNewGame(cols: number = 10, rows: number = 10, mines: number = 10): Observable<MinesweeperGame> {
    return this.http.get<MinesweeperGame>(`${this.baseUrl}game/new/${cols}/${rows}/${mines}`);
  }

  makeMove(point: Point): Observable<MinesweeperGame> {
    return this.http.post<MinesweeperGame>(`${this.baseUrl}game`, point);
  }

  toggleFlag(point: Point): Observable<MinesweeperGame> {
    return this.http.post<MinesweeperGame>(`${this.baseUrl}game/flag`, point);
  }
}
