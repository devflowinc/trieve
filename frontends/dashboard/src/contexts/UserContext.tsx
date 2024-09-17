import {
  Accessor,
  JSX,
  Resource,
  Show,
  createContext,
  createEffect,
  createResource,
  createSignal,
  useContext,
} from "solid-js";
import { createToast } from "../components/ShowToasts";
import { redirect, useSearchParams } from "@solidjs/router";
import { ApiContext } from "..";
import { DatasetAndUsage, SlimUser } from "trieve-ts-sdk";
import { OrgSelectPage } from "../pages/OrgSelect";

export interface UserStoreContextProps {
  children?: JSX.Element;
}

export interface Notification {
  message: string;
  type: "error" | "success" | "info";
  timeout?: number;
}

export interface UserStore {
  user: Accessor<SlimUser>;
  isNewUser: Accessor<boolean>;
  selectedOrg: Accessor<SlimUser["orgs"][0]>;
  setSelectedOrg: (orgId: string) => void;
  orgDatasets: Resource<DatasetAndUsage[]>;
  deselectOrg: () => void;
  login: () => void;
  logout: () => void;
}

export const UserContext = createContext<UserStore>({
  user: () => null as unknown as SlimUser,
  isNewUser: () => false,
  login: () => {},
  setSelectedOrg: () => {},
  orgDatasets: null as unknown as Resource<DatasetAndUsage[]>,
  deselectOrg: () => {},
  logout: () => {},
  selectedOrg: () => null as unknown as SlimUser["orgs"][0],
});

const getInitalUser = () => {
  const user = window.localStorage.getItem("trieve:user");
  if (user) {
    return JSON.parse(user) as SlimUser;
  }
};

export const UserContextWrapper = (props: UserStoreContextProps) => {
  const [searchParams] = useSearchParams();
  const trieve = useContext(ApiContext);

  const [user, setUser] = createSignal<SlimUser | null>(
    getInitalUser() ?? null,
  );
  const [isNewUser, setIsNewUser] = createSignal(false);
  const [selectedOrganization, setSelectedOrganization] = createSignal<
    SlimUser["orgs"][0] | null
  >(null);

  const apiHost = import.meta.env.VITE_API_HOST as string;

  const logout = () => {
    void fetch(`${apiHost}/auth?redirect_uri=${window.origin}`, {
      method: "DELETE",
      credentials: "include",
    }).then((res) => {
      res
        .json()
        .then((res) => {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
          window.location.href = res.logout_url;
          window.localStorage.removeItem("trieve:user");
          setUser(null);
          setSelectedOrganization(null);
        })
        .catch((error) => {
          console.error(error);
        });
    });
  };

  const login = () => {
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/`;
        }
        return res.json();
      })
      .then((data: SlimUser) => {
        // cache the user
        window.localStorage.setItem("trieve:user", JSON.stringify(data));

        // Grab org id from localstorage
        const possibleOrgId = window.localStorage.getItem(
          `${data.id}:selectedOrg`,
        );
        if (possibleOrgId) {
          const matchingOrg = data.orgs.find((org) => org.id === possibleOrgId);
          if (matchingOrg) {
            setSelectedOrganization(matchingOrg);
          }
        } else {
          const firstOrg = data.orgs.at(0);
          if (firstOrg) {
            setSelectedOrganization(firstOrg);
          } else {
            redirect("/dashboard/new_user");
          }
        }

        setUser(data);
      })
      .catch((err) => {
        setUser(null);
        console.error(err);
        createToast({
          title: "Error",
          type: "error",
          message: "Error logging in",
        });
      });
  };

  createEffect(() => {
    if (searchParams["new_user"]) {
      setIsNewUser(true);
    }
  });

  const [orgDatasets, _] = createResource(selectedOrganization, async (org) => {
    const result = await trieve.fetch(
      "/api/dataset/organization/{organization_id}",
      "get",
      {
        organizationId: org.id,
      },
    );
    return result;
  });

  const setSelectedOrg = (orgId: string) => {
    localStorage.setItem(`${user()?.id}:selectedOrg`, orgId);
    const org = user()?.orgs.find((org) => org.id === orgId);
    if (!org) {
      throw new Error("Organization not found");
    }
    setSelectedOrganization(org);
  };

  createEffect(() => {
    login();
  });

  const deselectOrg = () => {
    setSelectedOrganization(null);
  };

  return (
    <>
      <Show
        fallback={
          <div class="mt-4 flex min-h-full w-full items-center justify-center">
            <div class="mb-28 h-10 w-10 animate-spin rounded-full border-b-2 border-t-2 border-fuchsia-300" />
          </div>
        }
        when={user()}
      >
        {(user) => (
          <Show
            fallback={
              <OrgSelectPage selectOrg={setSelectedOrg} orgs={user().orgs} />
            }
            when={selectedOrganization()}
          >
            {(org) => (
              <UserContext.Provider
                value={{
                  user: user,
                  orgDatasets: orgDatasets,
                  deselectOrg,
                  selectedOrg: org,
                  setSelectedOrg: setSelectedOrg,
                  logout,
                  isNewUser: isNewUser,
                  login,
                }}
              >
                {props.children}
                <Show when={isNewUser()}>
                  <div>New user!!</div>
                </Show>
              </UserContext.Provider>
            )}
          </Show>
        )}
      </Show>
    </>
  );
};
