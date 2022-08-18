import { Component, Inject, ViewChild, ElementRef, AfterViewInit } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
  selector: 'app-minesweeper-board',
  templateUrl: './minesweeper-board.component.html'
})
export class MinesweeperBoardComponent implements AfterViewInit {
  public minesweeperGame: MinesweeperGame | null = null;
  private cellSize: number = 10;
  private id: number = 0;
  private imageLookup: Record<number, HTMLImageElement> = {};
  @ViewChild('canvas', {static: false}) canvas: ElementRef<HTMLCanvasElement> = {} as ElementRef;
  private images: string[] = [
    "/assets/svg/flag.svg",
    "/assets/svg/mine.svg",
    "/assets/svg/unknown.svg",
    "/assets/svg/0.svg",
    "/assets/svg/1.svg",
    "/assets/svg/2.svg",
    "/assets/svg/3.svg",
    "/assets/svg/4.svg",
    "/assets/svg/5.svg",
    "/assets/svg/6.svg",
    "/assets/svg/7.svg",
    "/assets/svg/8.svg"]

  constructor(private http: HttpClient, @Inject('BASE_URL') private baseUrl: string, private route: ActivatedRoute, private router: Router) {
  }

  async ngAfterViewInit(): Promise<void> {
    await this.initializeImageLookUp();
    this.canvas.nativeElement.addEventListener('click', (mouseEvent: MouseEvent) => this.onClicked(mouseEvent));
    this.canvas.nativeElement.addEventListener('contextmenu', (mouseEvent: MouseEvent) => this.onRightClicked(mouseEvent));
    this.canvas.nativeElement.oncontextmenu = () => {return false;};
    this.route.params.subscribe(params => {
      console.log("params updated:", params);
      this.id = Number(params["id"]);
      this.loadMinesweeperGame();
    });
  }

  private loadMinesweeperGame() {
    var url = this.baseUrl + 'game/' + this.id;
    if (this.id <= 0 || this.id === Number.NaN){
      url = this.baseUrl + 'game/new/10/10/10'
    }

    this.http.get<MinesweeperGame>(url).subscribe(result => {
      console.log("result:", result)
      this.minesweeperGame = result;
      if (this.id <= 0 || this.id === Number.NaN){
        this.router.navigate(['minesweeper/' + this.minesweeperGame.id]);
      }
    },
    error => {
      console.error(error);
      this.minesweeperGame = null;
    },
    async () => {
      if(!this.minesweeperGame) return;
      this.drawBoard()
    }
    );
  }

  public drawBoard(){
    if (!this.minesweeperGame) return;
    this.minesweeperGame
    const columnCount = this.minesweeperGame.board.length;
    const rowCount = this.minesweeperGame.board[0].length;
    const canvas = this.canvas.nativeElement;
    canvas.width = Math.max(24*columnCount, 420);
    canvas.height = canvas.width;
    this.cellSize = canvas.width/columnCount;
    const context = canvas.getContext("2d");
    if (context === null) return;
    context.beginPath();
    context.fillStyle = 'black';
    context.fillRect(0, 0, canvas.width, canvas.height)
    for (var x = 0; x < columnCount; ++x){
      for (var y = 0; y < rowCount; ++y){
        let cellNumber = this.minesweeperGame.board[x][y]
        context.drawImage(this.imageLookup[cellNumber], this.cellSize*x+1, this.cellSize*y+1, this.cellSize-2, this.cellSize-2);
      }
    }

    var gameState = this.GetGameState();

    if(gameState === GameState.MineHit) {
      
      context.fillStyle = 'rgba(0,0,0,0.3)';
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.fillStyle = 'white';
      context.font = '90px calibri';
      context.textAlign = 'center';
      context.fillText("Game Over", canvas.width/2, canvas.height/2, canvas.width);
    } else if (gameState === GameState.Victory) {
      context.fillStyle = 'rgba(0,0,0,0.3)';
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.fillStyle = 'white';
      context.font = '90px calibri';
      context.textAlign = 'center';
      context.fillText("VICTORY!", canvas.width/2, canvas.height/2, canvas.width);
    }

  }
  
  public onClicked(mouseEvent: MouseEvent) {
    console.log(mouseEvent);
    var point = this.getPointFromMouseEvent(mouseEvent);
    console.log(point);
    if (!this.minesweeperGame) return;
    this.http.post<MinesweeperGame>(this.baseUrl + 'game/' + this.minesweeperGame.id, point).subscribe(result => {
      this.minesweeperGame = result;
    },
    error => console.error(error),
    () => this.drawBoard()
    );
  }

  public onRightClicked(mouseEvent: MouseEvent) {
    console.log(mouseEvent);
    var point = this.getPointFromMouseEvent(mouseEvent);
    console.log(point);
    if (!this.minesweeperGame) return;
    this.http.post<MinesweeperGame>(this.baseUrl + 'game/flag/' + this.minesweeperGame.id, point).subscribe(result => {
      this.minesweeperGame = result;
    },
    error => console.error(error),
    () => this.drawBoard()
    );
  }

  private getPointFromMouseEvent(mouseEvent: MouseEvent): Point {
    var x = Math.floor(mouseEvent.offsetX/this.cellSize);
    var y = Math.floor(mouseEvent.offsetY/this.cellSize);
    return new Point(x, y);
  }

  private GetGameState(): GameState {
    if (!this.minesweeperGame) return GameState.Running;
    var unknownCount = 0;
    for (var x = 0; x < this.minesweeperGame.board.length; x++) {
      for (var y = 0; y < this.minesweeperGame.board[0].length; y++){
        if (this.minesweeperGame.board[x][y] === -1 || this.minesweeperGame.board[x][y] === -3){
          ++unknownCount;
        } else if (this.minesweeperGame.board[x][y] === -2){
          return GameState.MineHit;
        }
        if (unknownCount > this.minesweeperGame.mineCount) {
          return GameState.Running;
        }
      }
    }
    return unknownCount === this.minesweeperGame.mineCount ? GameState.Victory : GameState.Running;
  }

  private async initializeImageLookUp() {
    for(var i = 0; i < this.images.length; i++)
      this.imageLookup[i-3] = await this.loadImage(this.images[i]);
  }

  private async loadImage(src: string): Promise<HTMLImageElement> {
    const image = new Image();
    image.src = src;
    return new Promise(resolve => {
        image.onload = (ev) => {
            resolve(image);
        }
    });
  }
}

interface MinesweeperGame {
  id: number;
  board: number[][];
  mineCount: number;
  flagPoints: Point[];
}

class Point {
  constructor(public x : number, public y: number){ }
}

enum GameState {
  Running,
  MineHit,
  Victory
}

