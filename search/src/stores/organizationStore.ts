import { atom } from "nanostores";
import { persistentAtom } from "@nanostores/persistent";
import { isOrganizationDTO, type OrganizationDTO } from "../../utils/apiTypes";
import { currentUser } from "./userStore";

const tryParse = (encoded: string) => {
  try {
    if (isOrganizationDTO(JSON.parse(encoded))) {
      return JSON.parse(encoded) as OrganizationDTO;
    } else {
      return null;
    }
  } catch (e) {
    return null;
  }
};

export const currentOrganization = persistentAtom("currentOrganization", null, {
  encode: JSON.stringify,
  decode: tryParse,
});
export const organizations = atom<OrganizationDTO[]>([]);

currentUser.subscribe((user) => {
  if (!user) {
    return;
  }
  organizations.set(user.orgs);
  currentOrganization.set(user.orgs[0]);
});
