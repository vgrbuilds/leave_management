import { Injectable } from '@angular/core';
import { Observable, of, delay, catchError } from 'rxjs';
import {
  User, UserRole, LeaveRequest, LeaveBalance, LeavePolicy, Holiday,
  LeaveType, LeaveStatus, LeaveDuration, LeaveCalendarEntry, DashboardStats, ComplianceReport,
  ApiResponse, PaginatedResponse, AuthResponse
} from '../models';

import { AuthService } from './auth.service';
import { LeaveService } from './leave.service';
import { UserService } from './user.service';

/**
 * MockDataService acts as a smart proxy.
 * It attempts to fetch data from the real Rust backend via the actual services.
 * If the connection fails (e.g., server offline), it catches the error and
 * seamlessly falls back to returning the stub mock data.
 */
@Injectable({
  providedIn: 'root',
})
export class MockDataService {
  constructor(
    private auth: AuthService,
    private leave: LeaveService,
    private user: UserService
  ) {}

  // ─── Stub Data for Fallbacks ─────────────────────────
  private users: User[] = [
    {
      id: '1', employeeId: 'EMP001', firstName: 'John', lastName: 'Doe',
      email: 'john.doe@company.com', password: 'password123', role: UserRole.EMPLOYEE, department: 'Engineering',
      designation: 'Software Engineer', joiningDate: '2022-01-15', isActive: true,
    },
    {
      id: '2', employeeId: 'EMP002', firstName: 'Jane', lastName: 'Smith',
      email: 'jane.smith@company.com', password: 'password123', role: UserRole.MANAGER, department: 'Engineering',
      designation: 'Engineering Manager', joiningDate: '2020-06-01', isActive: true,
    },
    {
      id: '3', employeeId: 'EMP003', firstName: 'Sarah', lastName: 'Johnson',
      email: 'sarah.johnson@company.com', password: 'password123', role: UserRole.HR_ADMIN, department: 'Human Resources',
      designation: 'HR Manager', joiningDate: '2019-03-10', isActive: true,
    },
  ];

  private leaveRequests: LeaveRequest[] = [];
  private leaveBalances: LeaveBalance[] = [];
  private holidays: Holiday[] = [];
  private leavePolicies: LeavePolicy[] = [];

  // ─── Utility Methods ─────────────────────────────────

  private fallback<T>(data: T, message = 'Success (Mock Data)'): Observable<ApiResponse<T>> {
    console.warn('Backend unavailable, falling back to mock data:', message);
    return of({
      success: true,
      data,
      message,
      timestamp: new Date().toISOString(),
    }).pipe(delay(300));
  }

  private emptyPage<T>(): PaginatedResponse<T> {
    return { items: [], totalItems: 0, currentPage: 1, totalPages: 0, pageSize: 10 };
  }

  // ─── Proxied Methods with Fallback ───────────────────

  mockLogin(email: string, password: string): Observable<ApiResponse<AuthResponse>> {
    return this.auth.login({ email, password }).pipe(
      catchError(() => {
        const u = this.users.find((x) => x.email === email) || this.users[0];
        return this.fallback<AuthResponse>({
          token: 'mock-token-' + u.id,
          refreshToken: 'mock-refresh-' + u.id,
          expiresIn: 86400,
          user: u,
        }, 'Login successful (Mock)');
      })
    );
  }

  mockDashboardStats(): Observable<ApiResponse<DashboardStats>> {
    return this.user.getDashboardStats().pipe(
      catchError(() => this.fallback<DashboardStats>({
        totalEmployees: 3, pendingRequests: 0, approvedToday: 0, onLeaveToday: 0, upcomingHolidays: 0
      }))
    );
  }

  mockGetMyLeaves(_employeeId: string): Observable<ApiResponse<PaginatedResponse<LeaveRequest>>> {
    return this.leave.getMyLeaves().pipe(
      catchError(() => this.fallback(this.emptyPage<LeaveRequest>()))
    );
  }

  mockGetLeaveBalance(): Observable<ApiResponse<LeaveBalance[]>> {
    return this.leave.getMyBalance().pipe(
      catchError(() => this.fallback(this.leaveBalances))
    );
  }

  mockGetPendingApprovals(): Observable<ApiResponse<PaginatedResponse<LeaveRequest>>> {
    return this.leave.getPendingApprovals().pipe(
      catchError(() => this.fallback(this.emptyPage<LeaveRequest>()))
    );
  }

  mockGetAllLeaveRequests(): Observable<ApiResponse<PaginatedResponse<LeaveRequest>>> {
    return this.leave.getAllLeaveRequests().pipe(
      catchError(() => this.fallback(this.emptyPage<LeaveRequest>()))
    );
  }

  mockGetHolidays(): Observable<ApiResponse<Holiday[]>> {
    return this.leave.getHolidays().pipe(
      catchError(() => this.fallback(this.holidays))
    );
  }

  mockGetPolicies(): Observable<ApiResponse<LeavePolicy[]>> {
    return this.leave.getLeavePolicies().pipe(
      catchError(() => this.fallback(this.leavePolicies))
    );
  }

  mockGetCalendarEntries(): Observable<ApiResponse<LeaveCalendarEntry[]>> {
    const d = new Date();
    return this.leave.getCalendarEntries(d.getMonth() + 1, d.getFullYear()).pipe(
      catchError(() => this.fallback<LeaveCalendarEntry[]>([]))
    );
  }

  mockGetAllUsers(): Observable<ApiResponse<PaginatedResponse<User>>> {
    return this.user.getAllUsers().pipe(
      catchError(() => this.fallback(this.emptyPage<User>()))
    );
  }

  mockGetComplianceReport(): Observable<ApiResponse<ComplianceReport[]>> {
    return this.user.getComplianceReport().pipe(
      catchError(() => this.fallback<ComplianceReport[]>([]))
    );
  }

  mockApplyLeave(leaveRequest: any): Observable<ApiResponse<LeaveRequest>> {
    // leaveRequest here might be Partial<LeaveRequest> from component,
    // we map it to LeaveApplyPayload format for the real service.
    const payload = {
      leaveType: leaveRequest.leaveType || LeaveType.ANNUAL,
      startDate: leaveRequest.startDate || '',
      endDate: leaveRequest.endDate || '',
      duration: leaveRequest.duration || LeaveDuration.FULL_DAY,
      reason: leaveRequest.reason || '',
    };

    return this.leave.applyLeave(payload).pipe(
      catchError(() => {
        const nr: LeaveRequest = {
          id: 'LR999', employeeId: leaveRequest.employeeId || '1', employeeName: 'Mock User',
          department: leaveRequest.department || 'Mock', leaveType: payload.leaveType,
          startDate: payload.startDate, endDate: payload.endDate, duration: payload.duration,
          totalDays: leaveRequest.totalDays || 1, reason: payload.reason,
          status: LeaveStatus.PENDING, appliedOn: new Date().toISOString().split('T')[0],
        };
        this.leaveRequests.push(nr);
        return this.fallback(nr, 'Leave applied (Mock)');
      })
    );
  }

  mockApproveReject(id: string, action: 'APPROVE' | 'REJECT', comments?: string): Observable<ApiResponse<LeaveRequest>> {
    return this.leave.approveOrRejectLeave({ leaveRequestId: id, action, comments }).pipe(
      catchError(() => {
        const req = this.leaveRequests.find(r => r.id === id);
        if (req) {
          req.status = action === 'APPROVE' ? LeaveStatus.APPROVED : LeaveStatus.REJECTED;
          req.rejectionReason = comments;
        }
        return this.fallback(req as LeaveRequest, `Leave ${action.toLowerCase()}d (Mock)`);
      })
    );
  }
}
