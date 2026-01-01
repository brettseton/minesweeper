import { Component, AfterViewInit, OnDestroy, ChangeDetectionStrategy, ChangeDetectorRef, signal, computed, inject } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { Subscription } from 'rxjs';
import { distinctUntilChanged } from 'rxjs/operators';

import { GameStateService } from '../game-state.service';
import { MinesweeperApiService } from './minesweeper-api.service';
import { MinesweeperGame, Point, GameState, BoardCell } from './models';

@Component({
  selector: 'app-minesweeper-board',
  templateUrl: './minesweeper-board.component.html',
  styleUrls: ['./minesweeper-board.component.css'],
  standalone: true,
  imports: [],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class MinesweeperBoardComponent implements AfterViewInit, OnDestroy {
  public minesweeperGame = signal<MinesweeperGame | null>(null);
  public rows = computed(() => {
    const game = this.minesweeperGame();
    if (!game || !game.board || game.board.length === 0) return [];
    
    const numCols = game.board.length;
    const numRows = game.board[0].length;
    const transposed: number[][] = [];
    
    for (let y = 0; y < numRows; y++) {
      const row: number[] = [];
      for (let x = 0; x < numCols; x++) {
        row.push(game.board[x][y]);
      }
      transposed.push(row);
    }
    return transposed;
  });

  public columnCount = computed(() => this.minesweeperGame()?.board?.length ?? 0);
  public rowCount = computed(() => this.minesweeperGame()?.board?.[0]?.length ?? 0);

  private id = 0;
  private subscription = new Subscription();

  private router = inject(Router);
  private route = inject(ActivatedRoute);
  private gameStateService = inject(GameStateService);
  private apiService = inject(MinesweeperApiService);
  private cdr = inject(ChangeDetectorRef);

  ngOnDestroy(): void {
    this.subscription.unsubscribe();
  }

  async ngAfterViewInit(): Promise<void> {
    this.handleInitialId();
    this.subscribeToGameChanges();
  }

  private handleInitialId() {
    const idParam = this.route.snapshot.params['id'];
    const numericId = Number(idParam);
    if (idParam && !isNaN(numericId) && numericId !== 0) {
      this.gameStateService.setActiveGame(numericId);
      this.router.navigate(['/minesweeper'], { replaceUrl: true });
    }
  }

  private subscribeToGameChanges() {
    this.subscription.add(
      this.gameStateService.activeGameId$
        .pipe(distinctUntilChanged())
        .subscribe(activeId => {
          this.id = activeId || 0;
          this.loadMinesweeperGame();
        })
    );
  }

  private loadMinesweeperGame() {
    const request = this.id > 0 
      ? this.apiService.getGame(this.id) 
      : this.apiService.getNewGame();

    request.subscribe({
      next: (result) => {
        this.minesweeperGame.set(result);
        if (this.id !== result.id) {
          this.id = result.id;
          this.gameStateService.setActiveGame(result.id);
          this.router.navigate(['/minesweeper'], { replaceUrl: true });
        }
        this.cdr.markForCheck();
      },
      error: (err) => {
        console.error(err);
        this.minesweeperGame.set(null);
        if (this.id !== 0) {
          this.gameStateService.setActiveGame(null);
        }
        this.cdr.markForCheck();
      }
    });
  }

  public newGame() {
    this.gameStateService.setActiveGame(null);
  }

  public onCellClicked(x: number, y: number) {
    const game = this.minesweeperGame();
    if (!game || this.isGameOver()) return;

    this.apiService.makeMove({ x, y, gameId: game.id }).subscribe({
      next: (res) => { 
        this.minesweeperGame.set(res); 
        this.cdr.markForCheck();
      },
      error: (err) => console.error(err)
    });
  }

  public onCellRightClicked(event: MouseEvent, x: number, y: number) {
    event.preventDefault();
    const game = this.minesweeperGame();
    if (!game || this.isGameOver()) return;

    this.apiService.toggleFlag({ x, y, gameId: game.id }).subscribe({
      next: (res) => { 
        this.minesweeperGame.set(res); 
        this.cdr.markForCheck();
      },
      error: (err) => console.error(err)
    });
  }

  public getCellImage(value: number): string {
    const mapping: Record<number, string> = {
      [-3]: 'flag',
      [-2]: 'mine',
      [-1]: 'unknown',
      [0]: '0',
      [1]: '1',
      [2]: '2',
      [3]: '3',
      [4]: '4',
      [5]: '5',
      [6]: '6',
      [7]: '7',
      [8]: '8'
    };
    return `/assets/svg/${mapping[value]}.svg`;
  }

  public getGameState(): GameState {
    const game = this.minesweeperGame();
    if (!game) return GameState.Running;
    
    let unknownCount = 0;
    const board = game.board;

    for (let x = 0; x < board.length; x++) {
      for (let y = 0; y < board[0].length; y++) {
        const cell = board[x][y];
        if (cell === BoardCell.MINE) return GameState.MineHit;
        if (cell === BoardCell.UNKNOWN || cell === BoardCell.FLAG) unknownCount++;
      }
    }
    
    return unknownCount === game.mineCount ? GameState.Victory : GameState.Running;
  }

  public isGameOver(): boolean {
    return this.getGameState() !== GameState.Running;
  }

  public getStatusText(): string {
    const state = this.getGameState();
    if (state === GameState.MineHit) return 'Game Over';
    if (state === GameState.Victory) return 'VICTORY!';
    return '';
  }
}

