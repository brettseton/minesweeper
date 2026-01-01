import { Injectable, signal, computed, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';

export interface AuthStatus {
  isAuthenticated: boolean;
  name?: string;
  loading: boolean;
}

interface RawAuthStatus {
  isAuthenticated?: boolean;
  IsAuthenticated?: boolean;
  name?: string;
  Name?: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private statusSignal = signal<AuthStatus>({ isAuthenticated: false, loading: true });
  public status = computed(() => this.statusSignal());

  private http = inject(HttpClient);

  constructor() {
    console.log('AuthService initialized');
    this.checkStatus();
  }

  checkStatus(): void {
    console.log('Checking auth status...');
    this.http.get<RawAuthStatus>('/account/status').subscribe({
      next: (rawStatus) => {
        console.log('Raw Auth status from backend:', rawStatus);
        
        const isAuth = rawStatus.isAuthenticated === true || rawStatus.IsAuthenticated === true;
        
        this.statusSignal.set({
          isAuthenticated: isAuth,
          name: rawStatus.name || rawStatus.Name,
          loading: false
        });
        console.log('Mapped Auth status:', this.status());
      },
      error: (error) => {
        console.error('Auth status check failed:', error);
        this.statusSignal.set({ isAuthenticated: false, loading: false });
      }
    });
  }

  login(): void {
    const loginUrl = `/account/google-login?ReturnURL=${encodeURIComponent(window.location.origin)}`;
    console.log('Redirecting to login:', loginUrl);
    window.location.href = loginUrl;
  }

  logout(): void {
    this.http.post('/account/google-logout', {}).subscribe(() => {
      this.statusSignal.set({ isAuthenticated: false, loading: false });
      window.location.reload();
    });
  }
}

