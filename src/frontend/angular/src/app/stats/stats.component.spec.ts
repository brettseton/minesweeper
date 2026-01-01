import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { provideHttpClient } from '@angular/common/http';
import { StatsComponent, GameStats } from './stats.component';

describe('StatsComponent', () => {
  let component: StatsComponent;
  let fixture: ComponentFixture<StatsComponent>;
  let httpMock: HttpTestingController;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StatsComponent],
      providers: [
        provideHttpClient(),
        provideHttpClientTesting()
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(StatsComponent);
    component = fixture.componentInstance;
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('should create', () => {
    fixture.detectChanges();
    httpMock.expectOne('/user/stats').flush({ won: 0, lost: 0, inProgress: 0 });
    expect(component).toBeTruthy();
  });

  it('should calculate win percentage correctly when data is loaded', () => {
    fixture.detectChanges(); // Trigger ngOnInit

    const mockStats: GameStats = { won: 3, lost: 1, inProgress: 2 };
    const req = httpMock.expectOne('/user/stats');
    req.flush(mockStats);

    // 3 wins out of 4 finished games (3+1) = 75%
    expect(component.winPercentage).toBe(75);
    expect(component.loading).toBeFalse();
    expect(component.getStrokeDashArray()).toBe('75 25');
  });

  it('should handle zero finished games', () => {
    fixture.detectChanges();

    const mockStats: GameStats = { won: 0, lost: 0, inProgress: 5 };
    const req = httpMock.expectOne('/user/stats');
    req.flush(mockStats);

    expect(component.winPercentage).toBe(0);
    expect(component.getStrokeDashArray()).toBe('0 100');
  });

  it('should show loading state initially', () => {
    fixture.detectChanges();
    const req = httpMock.expectOne('/user/stats');
    expect(component.loading).toBeTrue();
    const compiled = fixture.nativeElement as HTMLElement;
    expect(compiled.querySelector('.spinner-border')).toBeTruthy();
    req.flush({ won: 0, lost: 0, inProgress: 0 });
  });
});
