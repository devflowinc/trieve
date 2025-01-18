import {
  Accessor,
  Context,
  JSX,
  createContext,
  createEffect,
  createSignal,
  onMount,
} from "solid-js";
import {
  DatasetAndUsageDTO,
  OrganizationDTO,
  UserDTO,
  isDatasetAndUsageDTO,
  isUserDTO,
} from "../../utils/apiTypes";
import { createToast } from "../ShowToasts";
import { RouteSectionProps, useSearchParams } from "@solidjs/router";

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
  login: () => void;
  logout: () => void;
}

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
    login: () => {},
    logout: () => {},
  });

export const DatasetAndUserContextWrapper = (props: RouteSectionProps) => {
  const apiHost: string = import.meta.env.VITE_API_HOST as string;

  const [searchParams, setSearchParams] = useSearchParams();

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
    fetch(`${apiHost}/auth/me`, {
      credentials: "include",
    })
      .then((res) => {
        if (res.status === 401) {
          if (res.status === 401) {
            console.log(window.origin);
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

  onMount(() => {
    login();
  });

  createEffect(() => {
    const dataset = currentDataset();
    if (!dataset) {
      return;
    }

    setSearchParams({
      dataset: dataset.dataset.id,
    });

    localStorage.setItem("currentDataset", JSON.stringify(dataset));
  });

  createEffect(() => {
    let organization = selectedOrganization();

    if (!organization) {
      const paramsOrg = searchParams.organization;
      const storedOrganization = localStorage.getItem("currentOrganization");

      const user_orgs = user()?.orgs;

      if (user_orgs && user_orgs.length > 0) {
        organization = user_orgs[0];
        if (paramsOrg) {
          const foundParamsOrg = user_orgs.find((org) => org.id === paramsOrg);
          if (foundParamsOrg) {
            organization = foundParamsOrg;
          } else {
            window.history.replaceState({}, "", `${window.location.pathname}`);
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
      setSearchParams({ organization: organization.id });
      localStorage.setItem("currentOrganization", JSON.stringify(organization));
    }
  });

  createEffect(() => {
    const selectedOrg = selectedOrganization();

    if (!selectedOrg) {
      return;
    }

    void fetch(`${apiHost}/dataset/organization/${selectedOrg.id}`, {
      method: "GET",
      credentials: "include",
      headers: {
        "X-API-version": "2.0",
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

          const paramsDataset = searchParams.dataset;
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
          console.error(err);
        });
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
