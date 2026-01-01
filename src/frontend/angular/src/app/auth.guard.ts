import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { AuthService } from './auth.service';
import { map, take, filter } from 'rxjs/operators';
import { toObservable } from '@angular/core/rxjs-interop';

export const AuthGuard: CanActivateFn = (route, state) => {
  const authService = inject(AuthService);
  const router = inject(Router);

  console.log('AuthGuard checking access for:', state.url);
  return toObservable(authService.status).pipe(
    filter(status => !status.loading),
    take(1),
    map(status => {
      console.log('AuthGuard status decision:', status);
      if (status.isAuthenticated) {
        return true;
      }

      console.log('AuthGuard redirecting to login page');
      return router.parseUrl('/login');
    })
  );
};

