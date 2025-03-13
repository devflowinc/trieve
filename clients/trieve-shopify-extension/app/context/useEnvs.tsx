import { createContext, useContext } from "react";

type AppEnvs = {
  TRIEVE_BASE_URL: string;
};

const AppEnvContext = createContext<AppEnvs | null>(null);

export const useEnvs = () => {
  const context = useContext(AppEnvContext);
  if (!context) {
    throw new Error("useEnv must be used within AppEnvProvider");
  }
  return context;
};

export const AppEnvProvider = ({
  children,
  envs,
}: {
  children: React.ReactNode;
  envs: AppEnvs;
}) => {
  if (!envs) {
    return <div>Missing environment variables!!</div>;
  }
  return (
    <AppEnvContext.Provider value={envs}>{children}</AppEnvContext.Provider>
  );
};
