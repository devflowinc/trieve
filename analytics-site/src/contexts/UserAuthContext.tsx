import { SlimUser } from "shared/types";
import {
  Accessor,
  createContext,
  createSignal,
  onMount,
  ParentComponent,
  Show,
} from "solid-js";
import { apiHost } from "../utils/apiHost";
import { OrgContextProvider } from "./OrgContext";
import { TopBarLayout } from "../layouts/TopBarLayout";

export const UserContext = createContext<UserContextType>();

type UserContextType = {
  user: Accessor<SlimUser>;
};

export const UserAuthContextProvider: ParentComponent = (props) => {
  const [userInfo, setUserInfo] = createSignal<SlimUser | null>(null);

  const login = async () => {
    const response = await fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    });

    if (response.status === 401) {
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}`;
    }

    console.log("response", response)

    const userData = (await response.json()) as SlimUser;
    setUserInfo(userData);
  };

  onMount(() => {
    void login();
  });

  return (
    <>
      <Show when={userInfo()}>
        {(userInfo) => (
          <UserContext.Provider
            value={{
              user: userInfo,
            }}
          >
            <OrgContextProvider user={userInfo()}>
              <TopBarLayout>{props.children}</TopBarLayout>
            </OrgContextProvider>
          </UserContext.Provider>
        )}
      </Show>
    </>
  );
};
