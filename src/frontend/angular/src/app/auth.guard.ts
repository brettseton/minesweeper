import { Injectable } from '@angular/core';
import { CanActivate, ActivatedRouteSnapshot, RouterStateSnapshot, UrlTree, Router } from '@angular/router';
import { Observable } from 'rxjs';
import { AuthService } from './auth.service';
import { map, take, filter } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class AuthGuard implements CanActivate {
  constructor(private authService: AuthService, private router: Router) {}

  canActivate(
    route: ActivatedRouteSnapshot,
    state: RouterStateSnapshot
  ): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree {
    console.log('AuthGuard checking access for:', state.url);
    return this.authService.status$.pipe(
      filter(status => !status.loading),
      take(1),
      map(status => {
        console.log('AuthGuard status decision:', status);
        if (status.isAuthenticated) {
          return true;
        }

        console.log('AuthGuard redirecting to login page');
        // Redirect to local login page instead of immediate Google auth
        return this.router.parseUrl('/login');
      })
    );
  }
}

