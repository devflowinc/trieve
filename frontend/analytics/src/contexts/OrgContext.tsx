import { redirect, useSearchParams } from "@solidjs/router";
import { Organization, SlimUser } from "shared/types";
import {
  Accessor,
  Context,
  createContext,
  createSignal,
  ParentProps,
  Show,
} from "solid-js";

interface OrgDatasetContextProps extends ParentProps {
  user: SlimUser;
}

interface OrgContextType {
  selectedOrg: Accessor<Organization>;
  selectOrg: (orgId: string) => void;
}

export const OrgContext = createContext() as Context<OrgContextType>;

export const OrgContextProvider = (props: OrgDatasetContextProps) => {
  const [params, setParams] = useSearchParams();
  const getInitialUserOrg = () => {
    if (props.user.orgs.length === 0) {
      throw redirect("/error");
    }
    if (params.orgId) {
      return (
        props.user.orgs.find((org) => org.id === params.orgId) ||
        props.user.orgs[0]
      );
    }
    setParams({ orgId: props.user.orgs[0].id });
    return props.user.orgs[0];
  };

  const [selectedOrg, setSelectedOrg] =
    createSignal<Organization>(getInitialUserOrg());

  const setSelectedOrgWithParams = (orgId: string) => {
    const org = props.user.orgs.find((org) => org.id === orgId);
    if (!org) {
      throw redirect("/error");
    }
    setSelectedOrg(org);
    setParams({ orgId: org.id });
  };

  return (
    <>
      <OrgContext.Provider
        value={{
          selectedOrg,
          selectOrg: setSelectedOrgWithParams,
        }}
      >
        <Show when={selectedOrg}>{props.children}</Show>
      </OrgContext.Provider>
    </>
  );
};
