import { Injectable } from '@angular/core';
import { MinesweeperGame, GameState, BoardCell } from './models';

@Injectable({
  providedIn: 'root'
})
export class MinesweeperRendererService {
  private imageLookup: Record<number, HTMLImageElement> = {};
  private readonly images: string[] = [
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
    "/assets/svg/8.svg"
  ];

  async initialize(): Promise<void> {
    if (Object.keys(this.imageLookup).length > 0) return;
    
    const loaders = this.images.map(async (src, i) => {
      this.imageLookup[i - 3] = await this.loadImage(src);
    });
    await Promise.all(loaders);
  }

  draw(canvas: HTMLCanvasElement, game: MinesweeperGame, gameState: GameState): number {
    const columnCount = game.board.length;
    const rowCount = game.board[0].length;
    
    canvas.width = Math.max(24 * columnCount, 420);
    canvas.height = canvas.width;
    const cellSize = canvas.width / columnCount;
    
    const ctx = canvas.getContext("2d");
    if (!ctx) return cellSize;

    ctx.beginPath();
    ctx.fillStyle = 'black';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    for (let x = 0; x < columnCount; ++x) {
      for (let y = 0; y < rowCount; ++y) {
        const cellValue = game.board[x][y];
        ctx.drawImage(this.imageLookup[cellValue], cellSize * x + 1, cellSize * y + 1, cellSize - 2, cellSize - 2);
      }
    }

    if (gameState !== GameState.Running) {
      this.drawOverlay(ctx, canvas, gameState === GameState.MineHit ? "Game Over" : "VICTORY!");
    }

    return cellSize;
  }

  private drawOverlay(ctx: CanvasRenderingContext2D, canvas: HTMLCanvasElement, text: string) {
    ctx.fillStyle = 'rgba(0,0,0,0.3)';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = 'white';
    ctx.font = '90px calibri';
    ctx.textAlign = 'center';
    ctx.fillText(text, canvas.width / 2, canvas.height / 2, canvas.width);
  }

  private async loadImage(src: string): Promise<HTMLImageElement> {
    const image = new Image();
    image.src = src;
    return new Promise(resolve => {
      image.onload = () => resolve(image);
    });
  }
}
