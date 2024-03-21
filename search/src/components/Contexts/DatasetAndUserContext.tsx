import {
  Accessor,
  Context,
  JSX,
  createContext,
  createEffect,
  createSignal,
} from "solid-js";
import {
  ClientEnvsConfiguration,
  DatasetAndUsageDTO,
  OrganizationDTO,
  UserDTO,
  defaultClientEnvsConfiguration,
  isDatasetAndUsageDTO,
  isUserDTO,
} from "../../../utils/apiTypes";

export interface DatasetAndUserStoreContextProps {
  children: JSX.Element;
}

export interface Notification {
  message: string;
  type: "error" | "success" | "info";
  timeout?: number;
}

export interface DatasetAndUserStore {
  user: Accessor<UserDTO | null> | null;
  setUser: (user: UserDTO | null) => void;
  organizations: Accessor<OrganizationDTO[]> | null;
  currentOrganization: Accessor<OrganizationDTO | null> | null;
  setCurrentOrganization: (id: OrganizationDTO | null) => void;
  currentDataset: Accessor<DatasetAndUsageDTO | null> | null;
  setCurrentDataset: (dataset: DatasetAndUsageDTO | null) => void;
  datasetsAndUsages: Accessor<DatasetAndUsageDTO[]> | null;
  clientConfig: Accessor<ClientEnvsConfiguration>;
  login: () => void;
  logout: () => void;
}

const [clientConfig] = createSignal<ClientEnvsConfiguration>(
  defaultClientEnvsConfiguration,
);

export const DatasetAndUserContext: Context<DatasetAndUserStore> =
  createContext<DatasetAndUserStore>({
    user: null,
    setUser: () => {},
    organizations: null,
    currentOrganization: null,
    setCurrentOrganization: () => {},
    currentDataset: null,
    setCurrentDataset: () => {},
    datasetsAndUsages: null,
    clientConfig: clientConfig,
    login: () => {},
    logout: () => {},
  });

export const DatasetAndUserContextWrapper = (
  props: DatasetAndUserStoreContextProps,
) => {
  const [user, setUser] = createSignal<UserDTO | null>(null);
  const [selectedOrganization, setSelectedOrganization] =
    createSignal<OrganizationDTO | null>(null);
  const [organizations, setOrganizations] = createSignal<OrganizationDTO[]>([]);

  const [currentDataset, setCurrentDataset] =
    createSignal<DatasetAndUsageDTO | null>(null);
  const [datasetsAndUsages, setDatasetsAndUsages] = createSignal<
    DatasetAndUsageDTO[]
  >([]);
  const [clientConfig, setClientConfig] = createSignal<ClientEnvsConfiguration>(
    defaultClientEnvsConfiguration,
  );

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
      });
  };

  createEffect(() => {
    const dataset = currentDataset();
    if (dataset) {
      const params = new URL(window.location.href).searchParams;
      params.set("dataset", dataset.dataset.id);
      window.history.replaceState(
        {},
        "",
        `${window.location.pathname}?${params.toString()}`,
      );
      localStorage.setItem("currentDataset", JSON.stringify(dataset));
    }
  });

  createEffect(() => {
    let organization = selectedOrganization();
    if (!organization) {
      const user_orgs = user()?.orgs;
      if (user_orgs && user_orgs.length > 0) {
        organization = user_orgs[0];
        setSelectedOrganization(organization);
      }
    }

    if (organization) {
      const params = new URLSearchParams(window.location.search);
      if (params.get("organization") === null) {
        params.set("organization", organization.id);
        window.history.replaceState(
          {},
          "",
          `${window.location.pathname}?${params.toString()}`,
        );
        localStorage.setItem(
          "currentOrganization",
          JSON.stringify(organization),
        );
      }
    }
  });

  createEffect(() => {
    const api_host: string = import.meta.env.VITE_API_HOST as string;
    const organization = selectedOrganization();

    if (organization) {
      const params = new URLSearchParams(window.location.search);
      void fetch(`${api_host}/dataset/organization/${organization.id}`, {
        method: "GET",
        credentials: "include",
        headers: {
          "TR-Organization": organization.id,
        },
      }).then((res) => {
        void res
          .json()
          .then((data: unknown[]) => {
            if (data.every(isDatasetAndUsageDTO)) {
              if (data.length === 0) {
                setDatasetsAndUsages([]);
              }

              if (data.length > 0) {
                if (
                  currentDataset() === null &&
                  params.get("dataset") === null &&
                  localStorage.getItem("currentDataset") === null
                ) {
                  setCurrentDataset(data[0]);
                } else if (params.get("dataset") !== null) {
                  const dataset = data.find(
                    (d: DatasetAndUsageDTO) =>
                      d.dataset.id === params.get("dataset"),
                  );
                  if (dataset) {
                    setCurrentDataset(dataset);
                  } else {
                    setCurrentDataset(data[0]);
                  }
                } else if (localStorage.getItem("currentDataset") !== null) {
                  const dataset = JSON.parse(
                    localStorage.getItem("currentDataset") as string,
                  ) as DatasetAndUsageDTO;
                  setCurrentDataset(dataset);
                }
              }

              setDatasetsAndUsages(data);
            }
          })
          .catch((err) => {
            console.log(err);
          });
      });
    }
  });

  createEffect(() => {
    const dataset = currentDataset();
    if (dataset) {
      const apiHost: string = import.meta.env.VITE_API_HOST as string;
      void fetch(`${apiHost}/dataset/envs`, {
        method: "GET",
        credentials: "include",
        headers: {
          "TR-Dataset": dataset.dataset.id,
        },
      }).then((res) => {
        if (res.ok) {
          void res
            .json()
            .then((data) => {
              if (data) {
                setClientConfig(data);
              }
            })
            .catch((err) => {
              console.log(err);
            });
        }
      });
    }
  });

  createEffect(() => {
    login();
  });

  const datasetAndUserStore: DatasetAndUserStore = {
    user: user,
    setUser: setUser,
    organizations: organizations,
    currentOrganization: selectedOrganization,
    setCurrentOrganization: setSelectedOrganization,
    currentDataset: currentDataset,
    setCurrentDataset: setCurrentDataset,
    clientConfig: clientConfig,
    datasetsAndUsages: datasetsAndUsages,
    login: login,
    logout: () => {},
  };

  return (
    <DatasetAndUserContext.Provider value={datasetAndUserStore}>
      {props.children}
    </DatasetAndUserContext.Provider>
  );
};
