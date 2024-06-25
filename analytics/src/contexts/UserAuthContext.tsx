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
import { OrgDatasetContextProvider } from "./OrgDatasetContext";

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
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard/foo`;
    }
    const userData = await response.json();
    setUserInfo(userData);
  };

  onMount(async () => {
    login();
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
            <OrgDatasetContextProvider user={userInfo()}>
              {props.children}
            </OrgDatasetContextProvider>
          </UserContext.Provider>
        )}
      </Show>
    </>
  );
};
