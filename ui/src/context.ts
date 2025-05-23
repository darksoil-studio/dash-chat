import { AdminWebsocket } from '@holochain/client';
import { createContext } from '@lit/context';
import { Router } from '@darksoil-studio/holochain-elements';

export const rootRouterContext = createContext<Router>('router');

export const isMobileContext = createContext<boolean>('ismobile');

export const adminWebsocketContext =
	createContext<AdminWebsocket>('adminwebsocket');
