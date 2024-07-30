import {
  Accessor,
  JSX,
  Show,
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
  loading: Accessor<boolean> | null;
}

export const UserContext = createContext<UserStore>({
  user: null,
  setUser: () => {},
  selectedOrganizationId: null,
  setSelectedOrganizationId: () => {},
  loading: null,
  login: () => {},
  logout: () => {},
});

export const UserContextWrapper = (props: UserStoreContextProps) => {
  const apiHost = import.meta.env.VITE_API_HOST as string;

  const [user, setUser] = createSignal<SlimUser | null>(null);
  const [selectedOrganizationId, setSelectedOrganizationId] = createSignal<
    string | null
  >(null);
  const [isLoading, setIsLoading] = createSignal(false);

  const login = () => {
    setIsLoading(true);
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard/foo`;
        }
        return res.json();
      })
      .then((data) => {
        setUser(data);
      })
      .catch((err) => {
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error logging in",
        });
      })
      .finally(() => setIsLoading(false));
  };

  createEffect(() => {
    let organizationId = selectedOrganizationId();

    if (organizationId == null) {
      const path = window.location.pathname;
      const pathParts = path.split("/");
      const urlParams = new URLSearchParams(window.location.search);
      const orgId = urlParams.get("org") ?? pathParts[2];

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

    const userOrgIds = user()?.user_orgs.map((org) => org.organization_id);

    if (!userOrgIds?.includes(organizationId ?? "")) {
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
    loading: isLoading,
    login: login,
    logout: () => {},
  };

  return (
    <>
      <Show when={!isLoading()}>
        <UserContext.Provider value={userStore}>
          {props.children}
        </UserContext.Provider>
      </Show>
      <Show when={isLoading()}>
        <div class="mt-4 flex min-h-full w-full items-center justify-center">
          <div class="mb-28 h-10 w-10 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
        </div>
      </Show>
    </>
  );
};
