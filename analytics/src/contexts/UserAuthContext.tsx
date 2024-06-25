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

  onMount(async () => {
    // Fetch user info and throw to login if not correct
    const response = await fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    });
    if (response.status === 401) {
      window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}/dashboard/foo`;
    }
    const userData = await response.json();
    setUserInfo("user", userData);
  });

  return (
    <div>
      <div>User context</div>
      <Show when={userInfo.user}>
        <UserContext.Provider
          value={{
            user: userInfo.user!,
          }}
        >
          {props.children}
        </UserContext.Provider>
      </Show>
    </div>
  );
};
