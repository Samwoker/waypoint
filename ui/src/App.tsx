import React, { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom';
import { SendWebhookModal } from './components/common/SendWebhookModal';
import { CommandPalette } from './components/layout/CommandPalette';
import { Header } from './components/layout/Header';
import { Sidebar } from './components/layout/Sidebar';
import { ToastProvider } from './context/ToastContext';
import { DocsLandingPage } from './docs/pages/DocsLandingPage';
import { DocDetailPage } from './docs/pages/DocDetailPage';
import { LandingPage } from './pages/public/LandingPage';
import { PricingPage } from './pages/public/PricingPage';
import { FeaturesPage } from './pages/public/FeaturesPage';
import { ApiKeysPage } from './pages/ApiKeysPage';
import { AuthPage } from './pages/AuthPage';
import { DeliveriesPage } from './pages/DeliveriesPage';
import { DeliveryDetailPage } from './pages/DeliveryDetailPage';
import { DestinationDetailPage } from './pages/DestinationDetailPage';
import { DestinationsPage } from './pages/DestinationsPage';
import { DlqPage } from './pages/DlqPage';
import { EventDetailPage } from './pages/EventDetailPage';
import { EventsPage } from './pages/EventsPage';
import { OverviewPage } from './pages/OverviewPage';
import { SourceDetailPage } from './pages/SourceDetailPage';
import { SourcesPage } from './pages/SourcesPage';
import { StatisticsPage } from './pages/StatisticsPage';
import { SubscriptionDetailPage } from './pages/SubscriptionDetailPage';
import { SubscriptionsPage } from './pages/SubscriptionsPage';
import { TenantSettingsPage } from './pages/TenantSettingsPage';
import { TransformationsPage } from './pages/TransformationsPage';
import { UsagePage } from './pages/dashboard/UsagePage';
import { BillingPage } from './pages/dashboard/BillingPage';
import { useAppDispatch, useAppSelector } from './store/hooks';
import { checkAuthRequest } from './store/slices/authSlice';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, token, isLoading } = useAppSelector((state) => state.auth);
  const location = useLocation();

  if (isLoading) {
    return (
      <div className="h-screen bg-[#09090b] flex items-center justify-center text-xs font-mono text-zinc-500">
        Authenticating session...
      </div>
    );
  }

  if (!token && !user) {
    // Preserve requested target URL: e.g. /login?redirect=/dashboard/api-keys
    const redirectTarget = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?redirect=${redirectTarget}`} replace />;
  }

  return <>{children}</>;
}

function AuthenticatedDashboardLayout() {
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState<boolean>(false);
  const [isSendModalOpen, setIsSendModalOpen] = useState<boolean>(false);

  return (
    <div className="flex min-h-screen bg-[#09090b] text-zinc-100 font-sans antialiased">
      {/* Sidebar navigation */}
      <Sidebar onOpenCommandPalette={() => setIsCommandPaletteOpen(true)} />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0">
        <Header onOpenSendModal={() => setIsSendModalOpen(true)} />
        <main className="flex-1 overflow-y-auto bg-grid-pattern">
          <Routes>
            {/* OVERVIEW */}
            <Route
              path="/dashboard"
              element={
                <ProtectedRoute>
                  <OverviewPage onOpenSendModal={() => setIsSendModalOpen(true)} />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/overview"
              element={
                <ProtectedRoute>
                  <OverviewPage onOpenSendModal={() => setIsSendModalOpen(true)} />
                </ProtectedRoute>
              }
            />

            {/* INGESTION */}
            <Route
              path="/sources"
              element={
                <ProtectedRoute>
                  <SourcesPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/sources"
              element={
                <ProtectedRoute>
                  <SourcesPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/sources/:id"
              element={
                <ProtectedRoute>
                  <SourceDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/sources/:id"
              element={
                <ProtectedRoute>
                  <SourceDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/events"
              element={
                <ProtectedRoute>
                  <EventsPage onOpenSendModal={() => setIsSendModalOpen(true)} />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/events"
              element={
                <ProtectedRoute>
                  <EventsPage onOpenSendModal={() => setIsSendModalOpen(true)} />
                </ProtectedRoute>
              }
            />
            <Route
              path="/events/:id"
              element={
                <ProtectedRoute>
                  <EventDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/events/:id"
              element={
                <ProtectedRoute>
                  <EventDetailPage />
                </ProtectedRoute>
              }
            />

            {/* DELIVERY */}
            <Route
              path="/destinations"
              element={
                <ProtectedRoute>
                  <DestinationsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/destinations"
              element={
                <ProtectedRoute>
                  <DestinationsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/destinations/:id"
              element={
                <ProtectedRoute>
                  <DestinationDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/destinations/:id"
              element={
                <ProtectedRoute>
                  <DestinationDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/subscriptions"
              element={
                <ProtectedRoute>
                  <SubscriptionsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/subscriptions"
              element={
                <ProtectedRoute>
                  <SubscriptionsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/subscriptions/:id"
              element={
                <ProtectedRoute>
                  <SubscriptionDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/subscriptions/:id"
              element={
                <ProtectedRoute>
                  <SubscriptionDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/deliveries"
              element={
                <ProtectedRoute>
                  <DeliveriesPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/deliveries"
              element={
                <ProtectedRoute>
                  <DeliveriesPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/deliveries/:id"
              element={
                <ProtectedRoute>
                  <DeliveryDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/deliveries/:id"
              element={
                <ProtectedRoute>
                  <DeliveryDetailPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dlq"
              element={
                <ProtectedRoute>
                  <DlqPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/dlq"
              element={
                <ProtectedRoute>
                  <DlqPage />
                </ProtectedRoute>
              }
            />

            {/* SUBSCRIPTION, USAGE & BILLING */}
            <Route
              path="/usage"
              element={
                <ProtectedRoute>
                  <UsagePage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/usage"
              element={
                <ProtectedRoute>
                  <UsagePage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/billing"
              element={
                <ProtectedRoute>
                  <BillingPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/billing"
              element={
                <ProtectedRoute>
                  <BillingPage />
                </ProtectedRoute>
              }
            />

            {/* OBSERVABILITY */}
            <Route
              path="/stats"
              element={
                <ProtectedRoute>
                  <StatisticsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/stats"
              element={
                <ProtectedRoute>
                  <StatisticsPage />
                </ProtectedRoute>
              }
            />

            {/* DEVELOPER */}
            <Route
              path="/api-keys"
              element={
                <ProtectedRoute>
                  <ApiKeysPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/api-keys"
              element={
                <ProtectedRoute>
                  <ApiKeysPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/transformations"
              element={
                <ProtectedRoute>
                  <TransformationsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/transformations"
              element={
                <ProtectedRoute>
                  <TransformationsPage />
                </ProtectedRoute>
              }
            />

            {/* SETTINGS */}
            <Route
              path="/settings"
              element={
                <ProtectedRoute>
                  <TenantSettingsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/dashboard/settings"
              element={
                <ProtectedRoute>
                  <TenantSettingsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/tenants"
              element={
                <ProtectedRoute>
                  <TenantSettingsPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/tenants/settings"
              element={
                <ProtectedRoute>
                  <TenantSettingsPage />
                </ProtectedRoute>
              }
            />

            {/* Fallback */}
            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Routes>
        </main>
      </div>

      {/* Global Command Palette (Ctrl+K) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
      />

      {/* Send Test Webhook Modal */}
      <SendWebhookModal
        isOpen={isSendModalOpen}
        onClose={() => setIsSendModalOpen(false)}
      />
    </div>
  );
}

function MainAppRouter() {
  const dispatch = useAppDispatch();
  const location = useLocation();

  useEffect(() => {
    dispatch(checkAuthRequest());
  }, [dispatch]);

  const isPublicRoute =
    location.pathname === '/' ||
    location.pathname === '/pricing' ||
    location.pathname === '/features';

  const isDocsRoute = location.pathname.startsWith('/docs');
  const isAuthRoute =
    location.pathname === '/login' ||
    location.pathname === '/signup' ||
    location.pathname.startsWith('/auth');

  // 1. PUBLIC MARKETING WEBSITE
  if (isPublicRoute) {
    return (
      <Routes>
        <Route path="/" element={<LandingPage />} />
        <Route path="/pricing" element={<PricingPage />} />
        <Route path="/features" element={<FeaturesPage />} />
      </Routes>
    );
  }

  // 2. PUBLIC DOCUMENTATION PORTAL
  if (isDocsRoute) {
    return (
      <Routes>
        <Route path="/docs" element={<DocsLandingPage />} />
        <Route path="/docs/*" element={<DocDetailPage />} />
      </Routes>
    );
  }

  // 3. AUTHENTICATION PORTAL
  if (isAuthRoute) {
    return (
      <Routes>
        <Route path="/login" element={<AuthPage />} />
        <Route path="/signup" element={<AuthPage />} />
        <Route path="/auth/*" element={<AuthPage />} />
      </Routes>
    );
  }

  // 4. AUTHENTICATED CUSTOMER DASHBOARD
  return <AuthenticatedDashboardLayout />;
}

export default function App() {
  return (
    <BrowserRouter>
      <ToastProvider>
        <MainAppRouter />
      </ToastProvider>
    </BrowserRouter>
  );
}
