import React, { createContext, useContext, useEffect, useState } from 'react';
import { api } from '../api/client';
import { Tenant, User } from '../types';

interface AuthContextType {
  user: User | null;
  currentTenant: Tenant | null;
  tenants: Tenant[];
  token: string | null;
  isLoading: boolean;
  login: (email: string, pass: string) => Promise<void>;
  logout: () => void;
  switchTenant: (tenant: Tenant) => void;
  refreshTenants: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [tenants, setTenants] = useState<Tenant[]>([]);
  const [currentTenant, setCurrentTenant] = useState<Tenant | null>(null);
  const [token, setToken] = useState<string | null>(api.getToken());
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const refreshTenants = async () => {
    try {
      const list = await api.listTenants();
      setTenants(list);
      if (list.length > 0 && !currentTenant) {
        setCurrentTenant(list[0]);
      }
    } catch (e) {
      console.warn('Failed to load tenants:', e);
    }
  };

  useEffect(() => {
    const init = async () => {
      setIsLoading(true);
      try {
        if (token) {
          const u = await api.getMe();
          setUser(u);
          await refreshTenants();
        }
      } catch (e) {
        console.warn('Auth check failed, using guest mode:', e);
      } finally {
        setIsLoading(false);
      }
    };
    init();
  }, [token]);

  const login = async (email: string, pass: string) => {
    const res = await api.login(email, pass);
    setToken(res.access_token);
    const u = await api.getMe();
    setUser(u);
    await refreshTenants();
  };

  const logout = () => {
    api.setToken(null);
    api.setApiKey(null);
    setToken(null);
    setUser(null);
    setCurrentTenant(null);
  };

  const switchTenant = (tenant: Tenant) => {
    setCurrentTenant(tenant);
  };

  return (
    <AuthContext.Provider
      value={{
        user,
        currentTenant,
        tenants,
        token,
        isLoading,
        login,
        logout,
        switchTenant,
        refreshTenants,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};
