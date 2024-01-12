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
import { ViewCollection } from "./pages/collection/ViewCollection";
import { Settings } from "./pages/user/Settings";
import { ViewUser } from "./pages/user/ViewUser";

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
      <Route path="/collection/:id" component={ViewCollection} />
      <Route path="/user">
        <Route path="/settings" component={Settings} />
        <Route path="/:id" component={ViewUser} />
      </Route>
    </Router>
  ),
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  root!,
);
