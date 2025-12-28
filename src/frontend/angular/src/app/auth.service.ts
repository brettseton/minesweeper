import { Injectable, Inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject } from 'rxjs';
import { map, tap } from 'rxjs/operators';

export interface AuthStatus {
  isAuthenticated: boolean;
  name?: string;
  loading: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private statusSubject = new BehaviorSubject<AuthStatus>({ isAuthenticated: false, loading: true });
  public status$ = this.statusSubject.asObservable();

  constructor(private http: HttpClient) {
    console.log('AuthService initialized');
    this.checkStatus();
  }

  checkStatus(): void {
    console.log('Checking auth status...');
    this.http.get<any>('/account/status').subscribe(
      rawStatus => {
        console.log('Raw Auth status from backend:', rawStatus);
        
        // Explicitly check for true/false values
        const isAuth = rawStatus.isAuthenticated === true || rawStatus.IsAuthenticated === true;
        
        const status: AuthStatus = {
          isAuthenticated: isAuth,
          name: rawStatus.name || rawStatus.Name,
          loading: false
        };
        console.log('Mapped Auth status:', status);
        this.statusSubject.next(status);
      },
      error => {
        console.error('Auth status check failed:', error);
        this.statusSubject.next({ isAuthenticated: false, loading: false });
      }
    );
  }

  login(): void {
    const loginUrl = `/account/google-login?ReturnURL=${encodeURIComponent(window.location.origin)}`;
    console.log('Redirecting to login:', loginUrl);
    window.location.href = loginUrl;
  }

  logout(): void {
    this.http.post('/account/google-logout', {}).subscribe(() => {
      this.statusSubject.next({ isAuthenticated: false, loading: false });
      window.location.reload();
    });
  }
}

