import { signaliumPreset } from 'signalium/transform';
import {
	signaliumAsyncTransform,
	signaliumCallbackTransform,
	signaliumPromiseMethodsTransform,
} from 'signalium/transform';

console.log('aa');
const opts = {};
export default {
	// presets: [signaliumPreset()],
	plugins: [
		signaliumCallbackTransform({
			transformedImports: opts?.transformedImports ?? [],
			importPaths: opts?.importPaths,
			callbackImportPath: opts?.callbackImportPath,
		}),
		signaliumAsyncTransform({
			transformedImports: opts?.transformedImports ?? [],
			importPaths: opts?.importPaths,
		}),
		signaliumPromiseMethodsTransform({
			transformedImports: opts?.transformedImports ?? [],
			importPaths: opts?.importPaths,
			promiseImportPath: opts?.promiseImportPath,
		}),

		// {
		// 	visitor: {
		// 		Identifier(path) {
		// 			if (path.node && path.node.name) console.log(path.node.name);
		// 			// console.log(path)
		// 			// const name = path.node.name;
		// 			// // reverse the name: JavaScript -> tpircSavaJ
		// 			// path.node.name = name.split('').reverse().join('');
		// 		},
		// 	},
		// },
	],
};
