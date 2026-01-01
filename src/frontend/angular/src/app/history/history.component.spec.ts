import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideRouter } from '@angular/router';
import { HistoryComponent } from './history.component';
import { GameStateService } from '../game-state.service';
import { AuthService } from '../auth.service';
import { Router } from '@angular/router';
import { of } from 'rxjs';
import { signal } from '@angular/core';
import { BASE_URL } from '../base-url.token';

describe('HistoryComponent', () => {
  let component: HistoryComponent;
  let fixture: ComponentFixture<HistoryComponent>;
  let httpMock: HttpTestingController;
  let router: Router;

  const mockAuthService = {
    status: signal({ isAuthenticated: true, loading: false, name: 'Test User' }),
    checkStatus: jasmine.createSpy('checkStatus')
  };

  const mockGameStateService = {
    activeGameId$: of(null),
    setActiveGame: jasmine.createSpy('setActiveGame'),
    getActiveGameId: jasmine.createSpy('getActiveGameId').and.returnValue(null)
  };

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [
        HistoryComponent
      ],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting(),
        provideRouter([]),
        { provide: BASE_URL, useValue: 'http://localhost/' },
        { provide: AuthService, useValue: mockAuthService },
        { provide: GameStateService, useValue: mockGameStateService }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(HistoryComponent);
    component = fixture.componentInstance;
    httpMock = TestBed.inject(HttpTestingController);
    router = TestBed.inject(Router);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should create', () => {
    fixture.detectChanges();
    const req = httpMock.expectOne('/user/games');
    req.flush([]);
    expect(component).toBeTruthy();
  });

  it('should load games on init', () => {
    fixture.detectChanges();

    const mockGames = [
      { id: 101, status: 'Won', mineCount: 10 },
      { id: 102, status: 'InProgress', mineCount: 15 }
    ];

    const req = httpMock.expectOne('/user/games');
    expect(req.request.method).toBe('GET');
    req.flush(mockGames);

    expect(component.games.length).toBe(2);
    expect(component.loading).toBeFalse();
  });

  it('should handle error when fetching games', () => {
    spyOn(console, 'error');
    fixture.detectChanges();

    const req = httpMock.expectOne('/user/games');
    req.error(new ErrorEvent('network error'));

    expect(component.loading).toBeFalse();
    expect(console.error).toHaveBeenCalled();
  });

  it('should navigate and set active game when resumeGame is called', () => {
    fixture.detectChanges();
    const req = httpMock.expectOne('/user/games');
    req.flush([]);

    const routerSpy = spyOn(router, 'navigate');

    component.resumeGame(42);

    expect(mockGameStateService.setActiveGame).toHaveBeenCalledWith(42);
    expect(routerSpy).toHaveBeenCalledWith(['/minesweeper']);
  });
});
