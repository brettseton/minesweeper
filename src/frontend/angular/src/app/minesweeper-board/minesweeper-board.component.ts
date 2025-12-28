import { Component, ViewChild, ElementRef, AfterViewInit, OnDestroy } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { Subscription } from 'rxjs';
import { distinctUntilChanged } from 'rxjs/operators';

import { GameStateService } from '../game-state.service';
import { MinesweeperApiService } from './minesweeper-api.service';
import { MinesweeperRendererService } from './minesweeper-renderer.service';
import { MinesweeperGame, Point, GameState, BoardCell } from './models';

@Component({
  selector: 'app-minesweeper-board',
  templateUrl: './minesweeper-board.component.html'
})
export class MinesweeperBoardComponent implements AfterViewInit, OnDestroy {
  public minesweeperGame: MinesweeperGame | null = null;
  private cellSize: number = 10;
  private id: number = 0;
  private subscription: Subscription = new Subscription();

  @ViewChild('canvas', { static: false }) canvas!: ElementRef<HTMLCanvasElement>;

  constructor(
    private router: Router,
    private route: ActivatedRoute,
    private gameStateService: GameStateService,
    private apiService: MinesweeperApiService,
    private renderer: MinesweeperRendererService
  ) {}

  ngOnDestroy(): void {
    this.subscription.unsubscribe();
  }

  async ngAfterViewInit(): Promise<void> {
    await this.renderer.initialize();
    this.setupEventListeners();
    this.handleInitialId();
    this.subscribeToGameChanges();
  }

  private setupEventListeners() {
    const el = this.canvas.nativeElement;
    el.addEventListener('click', (e) => this.onClicked(e));
    el.addEventListener('contextmenu', (e) => this.onRightClicked(e));
    el.oncontextmenu = () => false;
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
        this.minesweeperGame = result;
        if (this.id !== result.id) {
          this.id = result.id;
          this.gameStateService.setActiveGame(result.id);
          this.router.navigate(['/minesweeper'], { replaceUrl: true });
        }
        this.drawBoard();
      },
      error: (err) => {
        console.error(err);
        this.minesweeperGame = null;
        if (this.id !== 0) {
          this.gameStateService.setActiveGame(null);
        }
      }
    });
  }

  public newGame() {
    this.gameStateService.setActiveGame(null);
  }

  public drawBoard() {
    if (!this.minesweeperGame) return;
    this.cellSize = this.renderer.draw(
      this.canvas.nativeElement, 
      this.minesweeperGame, 
      this.calculateGameState()
    );
  }

  public onClicked(mouseEvent: MouseEvent) {
    if (!this.minesweeperGame) return;
    const point = this.getPointFromMouseEvent(mouseEvent);
    point.gameId = this.minesweeperGame.id;

    this.apiService.makeMove(point).subscribe({
      next: (res) => { this.minesweeperGame = res; this.drawBoard(); },
      error: (err) => console.error(err)
    });
  }

  public onRightClicked(mouseEvent: MouseEvent) {
    if (!this.minesweeperGame) return;
    const point = this.getPointFromMouseEvent(mouseEvent);
    point.gameId = this.minesweeperGame.id;

    this.apiService.toggleFlag(point).subscribe({
      next: (res) => { this.minesweeperGame = res; this.drawBoard(); },
      error: (err) => console.error(err)
    });
  }

  private getPointFromMouseEvent(mouseEvent: MouseEvent): Point {
    return {
      x: Math.floor(mouseEvent.offsetX / this.cellSize),
      y: Math.floor(mouseEvent.offsetY / this.cellSize)
    };
  }

  private calculateGameState(): GameState {
    if (!this.minesweeperGame) return GameState.Running;
    
    let unknownCount = 0;
    const board = this.minesweeperGame.board;

    for (let x = 0; x < board.length; x++) {
      for (let y = 0; y < board[0].length; y++) {
        const cell = board[x][y];
        if (cell === BoardCell.MINE) return GameState.MineHit;
        if (cell === BoardCell.UNKNOWN || cell === BoardCell.FLAG) unknownCount++;
      }
    }
    
    return unknownCount === this.minesweeperGame.mineCount ? GameState.Victory : GameState.Running;
  }
}