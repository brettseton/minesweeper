import { Component, OnInit, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { DecimalPipe } from '@angular/common';

export interface GameStats {
  won: number;
  lost: number;
  inProgress: number;
}

@Component({
  selector: 'app-stats',
  templateUrl: './stats.component.html',
  styleUrls: ['./stats.component.css'],
  standalone: true,
  imports: [DecimalPipe]
})
export class StatsComponent implements OnInit {
  public stats: GameStats = { won: 0, lost: 0, inProgress: 0 };
  public loading = true;
  public winPercentage = 0;

  private http = inject(HttpClient);

  ngOnInit(): void {
    this.http.get<GameStats>('/user/stats').subscribe({
      next: (result) => {
        this.stats = result;
        const totalFinished = this.stats.won + this.stats.lost;
        this.winPercentage = totalFinished > 0 ? (this.stats.won / totalFinished) * 100 : 0;
        this.loading = false;
      },
      error: (error) => {
        console.error('Could not fetch stats', error);
        this.loading = false;
      }
    });
  }

  getStrokeDashArray(): string {
    // Circumference is exactly 100
    const won = Math.max(0, Math.min(100, this.winPercentage));
    return `${won} ${100 - won}`;
  }
}