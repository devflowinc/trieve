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
import { createToast } from "../ShowToast";

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

  const getQueryParam = (name: string) => {
    const urlParams = new URLSearchParams(window.location.search);
    return urlParams.get(name);
  };

  const setQueryParam = (name: string, value: string) => {
    const urlParams = new URLSearchParams(window.location.search);
    urlParams.set(name, value);
    const newUrl = `${window.location.pathname}?${urlParams.toString()}`;
    window.history.replaceState({}, "", newUrl);
  };

  const login = () => {
    const apiHost: string = import.meta.env.VITE_API_HOST as string;
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          window.location.href = `${apiHost}/auth?redirect_uri=${window.origin}`;
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
          const orgId = getQueryParam("organization");
          if (orgId) {
            const organization = data.orgs.find((org) => org.id === orgId);
            if (organization) {
              setSelectedOrganization(organization);
            }
          } else {
            setSelectedOrganization(data.orgs[0]);
          }
        }
      })
      .catch((err) => {
        console.error(err);
        createToast({
          type: "error",
          message: "Error logging in",
        });
      });
  };

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
              setDatasetsAndUsages(data);

              const datasetId = getQueryParam("dataset");
              const storedDataset = localStorage.getItem("currentDataset");

              if (datasetId !== null) {
                const foundParamsDataset = data.find(
                  (d) => d.dataset.id === datasetId,
                );
                if (foundParamsDataset) {
                  setCurrentDataset(foundParamsDataset);
                } else {
                  setCurrentDataset(data[0]);
                }
              } else if (storedDataset !== null) {
                const storedDatasetJson = JSON.parse(
                  storedDataset,
                ) as DatasetAndUsageDTO;
                if (
                  data.find(
                    (d) => d.dataset.id === storedDatasetJson.dataset.id,
                  )
                ) {
                  setCurrentDataset(storedDatasetJson);
                } else {
                  setCurrentDataset(data[0]);
                }
              } else {
                setCurrentDataset(data[0]);
              }
            }
          }
        })
        .catch((err) => {
          console.error("Error fetching datasets:", err);
        });
    }
  });

  createEffect(() => {
    const dataset = currentDataset();

    if (dataset) {
      setQueryParam("dataset", dataset.dataset.id);
      localStorage.setItem("currentDataset", JSON.stringify(dataset));
    }
  });

  createEffect(() => {
    const selectedOrg = selectedOrganization();

    if (selectedOrg) {
      setQueryParam("organization", selectedOrg.id);
      localStorage.setItem("currentOrganization", JSON.stringify(selectedOrg));
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
