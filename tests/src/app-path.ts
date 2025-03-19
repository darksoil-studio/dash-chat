import { dirname } from 'path';
import { fileURLToPath } from 'url';

export const appPath =
	dirname(fileURLToPath(import.meta.url)) + '/../../workdir/dash-chat.happ';

// export const oldAppPath =
// 	dirname(fileURLToPath(import.meta.url)) + '/../old-dash-chat.happ';

export const oldAppPath = appPath;
