import { Component, OnInit } from '@angular/core';
import { AuthService } from '../auth.service';
import { Router } from '@angular/router';
import { filter, take } from 'rxjs/operators';

@Component({
  selector: 'app-login',
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.css']
})
export class LoginComponent implements OnInit {

  constructor(private authService: AuthService, private router: Router) { }

  ngOnInit(): void {
    // If already logged in, go home
    this.authService.status$.pipe(
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

