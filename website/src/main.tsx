import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider, createBrowserRouter } from "react-router-dom";
import "./styles.css";
import App from "./App";
import Home from "./pages/Home";
import DocsPage from "./pages/DocsPage";
import NotFound from "./pages/NotFound";

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <App />,
      children: [
        { index: true, element: <Home /> },
        { path: "*", element: <DocsPage /> },
      ],
    },
    { path: "*", element: <NotFound /> },
  ],
  { basename: "/lintropy" },
);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);
