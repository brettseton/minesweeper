import { Component, OnInit, inject } from '@angular/core';
import { AuthService } from '../auth.service';
import { Router } from '@angular/router';
import { filter, take } from 'rxjs/operators';
import { toObservable } from '@angular/core/rxjs-interop';

@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.css'],
  standalone: true
})
export class LoginComponent implements OnInit {
  private authService = inject(AuthService);
  private router = inject(Router);

  ngOnInit(): void {
    // If already logged in, go home
    toObservable(this.authService.status).pipe(
      filter(status => !status.loading),
      take(1)
    ).subscribe(status => {
      if (status.isAuthenticated) {
        this.router.navigate(['/']);
      }
    });
  }

  loginWithGoogle(): void {
    this.authService.login();
  }
}

