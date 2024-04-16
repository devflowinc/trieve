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
import { createToast } from "../ShowToasts";

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
  const apiHost: string = import.meta.env.VITE_API_HOST as string;

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
        createToast({
          type: "error",
          message: "Error logging in",
        });
      });
  };

  createEffect(() => {
    login();
  });

  createEffect(() => {
    const dataset = currentDataset();
    if (!dataset) {
      return;
    }

    const params = new URL(window.location.href).searchParams;
    params.set("dataset", dataset.dataset.id);
    window.history.replaceState(
      {},
      "",
      `${window.location.pathname}?${params.toString()}`,
    );
    localStorage.setItem("currentDataset", JSON.stringify(dataset));
  });

  createEffect(() => {
    const params = new URLSearchParams(window.location.search);

    let organization = selectedOrganization();

    if (!organization) {
      const paramsOrg = params.get("organization");
      const storedOrganization = localStorage.getItem("currentOrganization");

      const user_orgs = user()?.orgs;

      if (user_orgs && user_orgs.length > 0) {
        organization = user_orgs[0];
        if (paramsOrg) {
          const foundParamsOrg = user_orgs.find((org) => org.id === paramsOrg);
          if (foundParamsOrg) {
            organization = foundParamsOrg;
          } else {
            window.history.pushState({}, "", `${window.location.pathname}`);
          }
        } else if (storedOrganization) {
          const storedOrgJson = JSON.parse(
            storedOrganization,
          ) as OrganizationDTO;

          if (user_orgs.find((org) => org.id === storedOrgJson?.id)) {
            organization = storedOrgJson;
          } else {
            localStorage.removeItem("currentOrganization");
          }
        }

        setSelectedOrganization(organization);
      }
    }

    if (organization) {
      const params = new URLSearchParams(window.location.search);

      params.set("organization", organization.id);
      window.history.pushState(
        {},
        "",
        `${window.location.pathname}?${params.toString()}`,
      );
      localStorage.setItem("currentOrganization", JSON.stringify(organization));
    }
  });

  createEffect(() => {
    const selectedOrg = selectedOrganization();

    if (!selectedOrg) {
      return;
    }

    const params = new URLSearchParams(window.location.search);
    void fetch(`${apiHost}/dataset/organization/${selectedOrg.id}`, {
      method: "GET",
      credentials: "include",
      headers: {
        "TR-Organization": selectedOrg.id,
      },
    }).then((res) => {
      void res
        .json()
        .then((data: unknown[]) => {
          if (!data.every(isDatasetAndUsageDTO)) {
            setDatasetsAndUsages([]);
            return;
          }

          if (data.length === 0) {
            setDatasetsAndUsages([]);
            return;
          }

          let newDataset = data[0];

          const paramsDataset = params.get("dataset");
          const storedDataset = localStorage.getItem("currentDataset");

          if (paramsDataset !== null) {
            const foundParamsDataset = data.find(
              (d) => d.dataset.id === paramsDataset,
            );
            if (foundParamsDataset) {
              newDataset = foundParamsDataset;
            }
          } else if (storedDataset !== null) {
            const storedDatasetJson = JSON.parse(
              storedDataset,
            ) as DatasetAndUsageDTO;
            if (
              data.find((d) => d.dataset.id === storedDatasetJson.dataset.id)
            ) {
              newDataset = storedDatasetJson;
            }
          }

          setCurrentDataset(newDataset);

          setDatasetsAndUsages(data);
        })
        .catch((err) => {
          console.log(err);
        });
    });
  });

  createEffect(() => {
    const dataset = currentDataset();
    if (!dataset) {
      return;
    }

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
