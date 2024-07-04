import {
  Accessor,
  JSX,
  createContext,
  createEffect,
  createSignal,
} from "solid-js";
import { createToast } from "../components/ShowToasts";
import { SlimUser } from "shared/types";

export interface UserStoreContextProps {
  children: JSX.Element;
}

export interface Notification {
  message: string;
  type: "error" | "success" | "info";
  timeout?: number;
}

export interface UserStore {
  user: Accessor<SlimUser | null> | null;
  setUser: (user: SlimUser | null) => void;
  selectedOrganizationId: Accessor<string | null> | null;
  setSelectedOrganizationId: (id: string | null) => void;
  login: () => void;
  logout: () => void;
}

export const UserContext = createContext<UserStore>({
  user: null,
  setUser: () => {},
  selectedOrganizationId: null,
  setSelectedOrganizationId: () => {},
  login: () => {},
  logout: () => {},
});

export const UserContextWrapper = (props: UserStoreContextProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;

  const [user, setUser] = createSignal<SlimUser | null>(null);
  const [selectedOrganizationId, setSelectedOrganizationId] = createSignal<
    string | null
  >(null);

  const login = () => {
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          // Redirect to SSO login
          window.location.href = `${apiHost}/auth/sso`;
          return;
        }
        return res.json();
      })
      .then((data) => {
        if (data) {
          setUser(data);
        }
      })
      .catch((err) => {
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error logging in",
        });
      });
  };

  createEffect((prev) => {
    let organizationId = selectedOrganizationId();

    if (organizationId == null) {
      const path = window.location.pathname;
      const pathParts = path.split("/");
      const orgId = pathParts[2];
      if (user()?.user_orgs) {
        const org = user()?.user_orgs.find(
          (org) => org.organization_id === orgId,
        );
        if (org) {
          organizationId = orgId;
          setSelectedOrganizationId(orgId);
        }
      }

      if (organizationId == null) {
        const user_orgs = user()?.user_orgs;
        if (user_orgs && user_orgs.length > 0) {
          organizationId = user_orgs[0].organization_id;
          setSelectedOrganizationId(organizationId);
        }
      }
    }

    if (prev !== organizationId) {
      login();
    }

    return organizationId;
  }, null);

  createEffect(() => {
    login();
  });

  const userStore: UserStore = {
    user: user,
    setUser: setUser,
    selectedOrganizationId: selectedOrganizationId,
    setSelectedOrganizationId: setSelectedOrganizationId,
    login: login,
    logout: () => {},
  };

  return (
    <UserContext.Provider value={userStore}>
      {props.children}
    </UserContext.Provider>
  );
};

