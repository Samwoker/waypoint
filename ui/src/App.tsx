import React, { useEffect, useState } from 'react';
import { BrowserRouter, Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { SendWebhookModal } from './components/common/SendWebhookModal';
import { CommandPalette } from './components/layout/CommandPalette';
import { Header } from './components/layout/Header';
import { Sidebar } from './components/layout/Sidebar';
import { ApiKeysPage } from './pages/ApiKeysPage';
import { AuthPage } from './pages/AuthPage';
import { DeliveriesPage } from './pages/DeliveriesPage';
import { DestinationsPage } from './pages/DestinationsPage';
import { DlqPage } from './pages/DlqPage';
import { DocsPage } from './pages/DocsPage';
import { EventsPage } from './pages/EventsPage';
import { OverviewPage } from './pages/OverviewPage';
import { SourcesPage } from './pages/SourcesPage';
import { SubscriptionsPage } from './pages/SubscriptionsPage';
import { TransformationsPage } from './pages/TransformationsPage';
import { useAppDispatch, useAppSelector } from './store/hooks';
import { checkAuthRequest } from './store/slices/authSlice';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { user, token, isLoading } = useAppSelector((state) => state.auth);

  if (isLoading) {
    return (
      <div className="h-screen bg-[#09090b] flex items-center justify-center text-xs font-mono text-zinc-500">
        Authenticating session...
      </div>
    );
  }

  if (!token && !user) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}

function AppLayout() {
  const dispatch = useAppDispatch();
  const location = useLocation();
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState<boolean>(false);
  const [isSendModalOpen, setIsSendModalOpen] = useState<boolean>(false);

  useEffect(() => {
    dispatch(checkAuthRequest());
  }, [dispatch]);

  const isAuthRoute = location.pathname === '/login' || location.pathname === '/register';

  if (isAuthRoute) {
    return (
      <Routes>
        <Route path="/login" element={<AuthPage />} />
        <Route path="/register" element={<AuthPage />} />
      </Routes>
    );
  }

  return (
    <div className="flex min-h-screen bg-[#09090b] text-zinc-100 font-sans antialiased">
      {/* Sidebar navigation */}
      <Sidebar onOpenCommandPalette={() => setIsCommandPaletteOpen(true)} />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0">
        <Header onOpenSendModal={() => setIsSendModalOpen(true)} />
        <main className="flex-1 overflow-y-auto bg-grid-pattern">
          <Routes>
            <Route
              path="/"
              element={
                <ProtectedRoute>
                  <OverviewPage onOpenSendModal={() => setIsSendModalOpen(true)} />
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
              path="/deliveries"
              element={
                <ProtectedRoute>
                  <DeliveriesPage />
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
              path="/sources"
              element={
                <ProtectedRoute>
                  <SourcesPage />
                </ProtectedRoute>
              }
            />
            <Route
              path="/destinations"
              element={
                <ProtectedRoute>
                  <DestinationsPage />
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
              path="/transformations"
              element={
                <ProtectedRoute>
                  <TransformationsPage />
                </ProtectedRoute>
              }
            />
            {/* Documentation is publicly accessible */}
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/docs/:section" element={<DocsPage />} />
            <Route
              path="/apikeys"
              element={
                <ProtectedRoute>
                  <ApiKeysPage />
                </ProtectedRoute>
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
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

export default function App() {
  return (
    <BrowserRouter>
      <AppLayout />
    </BrowserRouter>
  );
}
