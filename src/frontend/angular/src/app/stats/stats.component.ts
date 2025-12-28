import { Component, OnInit } from '@angular/core';
import { HttpClient } from '@angular/common/http';

export interface GameStats {
  won: number;
  lost: number;
  inProgress: number;
}

@Component({
  selector: 'app-stats',
  templateUrl: './stats.component.html',
  styleUrls: ['./stats.component.css']
})
export class StatsComponent implements OnInit {
  public stats: GameStats = { won: 0, lost: 0, inProgress: 0 };
  public loading = true;
  public winPercentage = 0;

  constructor(private http: HttpClient) { }

  ngOnInit(): void {
    this.http.get<GameStats>('/user/stats').subscribe(
      result => {
        this.stats = result;
        const totalFinished = this.stats.won + this.stats.lost;
        this.winPercentage = totalFinished > 0 ? (this.stats.won / totalFinished) * 100 : 0;
        this.loading = false;
      },
      error => {
        console.error('Could not fetch stats', error);
        this.loading = false;
      }
    );
  }

  getStrokeDashArray(): string {
    // Circumference is exactly 100
    const won = Math.max(0, Math.min(100, this.winPercentage));
    return `${won} ${100 - won}`;
  }
}

