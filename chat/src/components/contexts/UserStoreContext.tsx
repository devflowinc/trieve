import { Accessor, createContext, createSignal, JSX } from "solid-js";

export interface GlobalStoreProviderType {
  isLogin: Accessor<boolean> | null;
  setIsLogin: (isLogin: boolean) => void;
}

export const GlobalStoreContext = createContext<GlobalStoreProviderType>({
  isLogin: null,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  setIsLogin: (isLogin: boolean) => void 0,
});

export interface GlobalStoreProviderProps {
  children: JSX.Element;
}

const UserStoreContext = (props: GlobalStoreProviderProps) => {
  const [isLogin, setIsLogin] = createSignal<boolean>(false);

  const GlobalStoreProvider = {
    isLogin,
    setIsLogin,
  };

  return (
    <GlobalStoreContext.Provider value={GlobalStoreProvider}>
      {props.children}
    </GlobalStoreContext.Provider>
  );
};

export default UserStoreContext;
