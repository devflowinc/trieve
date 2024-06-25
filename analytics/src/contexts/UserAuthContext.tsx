import { SlimUser } from "shared/types";
import { createContext, onMount, ParentComponent, Show } from "solid-js";
import { createStore } from "solid-js/store";
import { apiHost } from "../utils/apiHost";

export const UserContext = createContext<UserContextType>();

type UserContextType = {
  user: SlimUser;
};

export const UserAuthContextProvider: ParentComponent = (props) => {
  const [userInfo, setUserInfo] = createStore({
    user: null,
  });

  const login = async () => {
    const response = await fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    });
    if (response.status === 401) {
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard/foo`;
    }
    const userData = await response.json();
    setUserInfo("user", userData);
  };

  onMount(async () => {
    login();
  });

  return (
    <>
      <Show when={userInfo.user}>
        <UserContext.Provider
          value={{
            user: userInfo.user!,
          }}
        >
          {props.children}
        </UserContext.Provider>
      </Show>
    </>
  );
};
