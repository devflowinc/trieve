/* @refresh reload */
import "./index.css";
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import { Home } from "./pages/Home";
import { Search } from "./pages/Search";
import { Upload } from "./pages/Upload";
import { CreateChunk } from "./pages/CreateChunk";
import { EditChunk } from "./pages/chunk/edit/EditChunk";
import { ViewChunk } from "./pages/chunk/ViewChunk";
import { ViewGroup } from "./pages/group/ViewGroup";
import { Settings } from "./pages/user/Settings";
import { OrgGroups } from "./pages/group/OrgGroups";
import { OrgFiles } from "./pages/file/OrgFiles";

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Route path="/" component={Home} />
      <Route path="/search" component={Search} />
      <Route path="/upload" component={Upload} />
      <Route path="/create" component={CreateChunk} />
      <Route path="/chunk">
        <Route path="/edit/:id" component={EditChunk} />
        <Route path="/:id" component={ViewChunk} />
      </Route>
      <Route path="/group" component={OrgGroups} />
      <Route path="/group/:id" component={ViewGroup} />
      <Route path="/file" component={OrgFiles} />
      <Route path="/user">
        <Route path="/settings" component={Settings} />
      </Route>
    </Router>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
