import { Injectable, signal, computed, inject } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { logs } from '@opentelemetry/api-logs';
import { SeverityNumber } from '@opentelemetry/api-logs';

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
  private logger = logs.getLogger('auth-service');

  private http = inject(HttpClient);

  constructor() {
    console.log('AuthService initialized');
    this.checkStatus();
  }

  checkStatus(): void {
    console.log('Checking auth status...');
    this.logger.emit({
      severityNumber: SeverityNumber.INFO,
      severityText: 'INFO',
      body: 'Checking auth status...',
    });

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

        this.logger.emit({
          severityNumber: SeverityNumber.INFO,
          severityText: 'INFO',
          body: 'Auth status check successful',
          attributes: {
            isAuthenticated: isAuth,
            userName: rawStatus.name || rawStatus.Name || 'anonymous'
          }
        });
      },
      error: (error) => {
        console.error('Auth status check failed:', error);
        this.logger.emit({
          severityNumber: SeverityNumber.ERROR,
          severityText: 'ERROR',
          body: 'Auth status check failed',
          attributes: {
            error: error.message
          }
        });
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

