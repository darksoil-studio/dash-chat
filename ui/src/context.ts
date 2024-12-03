import { createContext } from "@lit/context";
import { Router } from "@tnesh-stack/elements";

export const rootRouterContext = createContext<Router>("router");
