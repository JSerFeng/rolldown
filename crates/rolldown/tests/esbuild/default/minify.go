
func TestExportFormsWithMinifyIdentifiersAndNoBundle(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.js": `
				export default 123
				export var varName = 234
				export let letName = 234
				export const constName = 234
				function Func2() {}
				class Class2 {}
				export {Class as Cls, Func2 as Fn2, Class2 as Cls2}
				export function Func() {}
				export class Class {}
				export * from './a'
				export * as fromB from './b'
			`,
			"/b.js": "export default function() {}",
			"/c.js": "export default function foo() {}",
			"/d.js": "export default class {}",
			"/e.js": "export default class Foo {}",
		},
		entryPaths: []string{
			"/a.js",
			"/b.js",
			"/c.js",
			"/d.js",
			"/e.js",
		},
		options: config.Options{
			MinifyIdentifiers: true,
			AbsOutputDir:      "/out",
		},
	})
}


func TestImportFormsWithMinifyIdentifiersAndNoBundle(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import 'foo'
				import {} from 'foo'
				import * as ns from 'foo'
				import {a, b as c} from 'foo'
				import def from 'foo'
				import def2, * as ns2 from 'foo'
				import def3, {a2, b as c3} from 'foo'
				const imp = [
					import('foo'),
					function() { return import('foo') },
				]
				console.log(ns, a, c, def, def2, ns2, def3, a2, c3, imp)
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			MinifyIdentifiers: true,
			AbsOutputFile:     "/out.js",
		},
	})
}
