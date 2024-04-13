import {
  Accessor,
  JSX,
  createContext,
  createEffect,
  createSignal,
} from "solid-js";
import {
  DatasetAndUsageDTO,
  OrganizationDTO,
  UserDTO,
  isDatasetAndUsageDTO,
  isUserDTO,
} from "../../utils/apiTypes";

export interface UserStoreContextProps {
  children: JSX.Element;
}

export interface Notification {
  message: string;
  type: "error" | "success" | "info";
  timeout?: number;
}

export interface UserStore {
  user: Accessor<UserDTO | null> | null;
  setUser: (user: UserDTO | null) => void;
  organizations: Accessor<OrganizationDTO[]> | null;
  currentOrganization: Accessor<OrganizationDTO | null> | null;
  setCurrentOrganization: (id: OrganizationDTO | null) => void;
  currentDataset: Accessor<DatasetAndUsageDTO | null> | null;
  setCurrentDataset: (dataset: DatasetAndUsageDTO | null) => void;
  datasetsAndUsages: Accessor<DatasetAndUsageDTO[]> | null;
  login: () => void;
  logout: () => void;
}

export const UserContext = createContext<UserStore>({
  user: null,
  setUser: () => {},
  organizations: null,
  currentOrganization: null,
  setCurrentOrganization: () => {},
  currentDataset: null,
  setCurrentDataset: () => {},
  datasetsAndUsages: null,
  login: () => {},
  logout: () => {},
});

export const UserContextWrapper = (props: UserStoreContextProps) => {
  const [user, setUser] = createSignal<UserDTO | null>(null);
  const [selectedOrganization, setSelectedOrganization] =
    createSignal<OrganizationDTO | null>(null);
  const [organizations, setOrganizations] = createSignal<OrganizationDTO[]>([]);

  const [currentDataset, setCurrentDataset] =
    createSignal<DatasetAndUsageDTO | null>(null);
  const [datasetsAndUsages, setDatasetsAndUsages] = createSignal<
    DatasetAndUsageDTO[]
  >([]);

  const login = () => {
    const apiHost: string = import.meta.env.VITE_API_HOST as string;
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          if (res.status === 401) {
            window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}`;
          }
          setUser(null);
          setOrganizations([]);
          return null;
        }
        return res.json();
      })
      .then((data) => {
        if (isUserDTO(data)) {
          setUser(data);
          setOrganizations(data.orgs);
        }
      })
      .catch((err) => {
        console.error(err);
        const newEvent = new CustomEvent("show-toast", {
          detail: {
            type: "error",
            message: "Error logging in",
          },
        });
        window.dispatchEvent(newEvent);
      });
  };

  createEffect(() => {
    let organization = selectedOrganization();
    if (!organization) {
      const user_orgs = user()?.orgs;
      if (user_orgs && user_orgs.length > 0) {
        organization = user_orgs[0];
        setSelectedOrganization(organization);
      }
    }
  });

  createEffect(() => {
    const api_host: string = import.meta.env.VITE_API_HOST as string;
    const organization = selectedOrganization();

    if (organization) {
      void fetch(`${api_host}/dataset/organization/${organization.id}`, {
        method: "GET",
        credentials: "include",
        headers: {
          "TR-Organization": organization.id,
        },
      })
        .then((res) => {
          if (res.ok) {
            return res.json();
          }
        })
        .then((data) => {
          if (data && Array.isArray(data)) {
            if (data.length === 0) {
              setCurrentDataset(null);
              setDatasetsAndUsages([]);
            }
            if (data.length > 0 && data.every(isDatasetAndUsageDTO)) {
              setCurrentDataset(data[0]);
              setDatasetsAndUsages(data);
            }
          }
        });
    }
  });

  createEffect(() => {
    login();
  });

  const userStore: UserStore = {
    user: user,
    setUser: setUser,
    organizations: organizations,
    currentOrganization: selectedOrganization,
    setCurrentOrganization: setSelectedOrganization,
    currentDataset: currentDataset,
    setCurrentDataset: setCurrentDataset,
    datasetsAndUsages: datasetsAndUsages,
    login: login,
    logout: () => {},
  };

  return (
    <UserContext.Provider value={userStore}>
      {props.children}
    </UserContext.Provider>
  );
};
