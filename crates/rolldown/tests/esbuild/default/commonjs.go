
func TestSimpleCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				const fn = require('./foo')
				console.log(fn())
			`,
			"/foo.js": `
				module.exports = function() {
					return 123
				}
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

// This test makes sure that require() calls are still recognized in nested
// scopes. It guards against bugs where require() calls are only recognized in
// the top-level module scope.
func TestNestedCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				function nestedScope() {
					const fn = require('./foo')
					console.log(fn())
				}
				nestedScope()
			`,
			"/foo.js": `
				module.exports = function() {
					return 123
				}
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

// This test makes sure that NewExpressions containing require() calls aren't
// broken.
func TestNewExpressionCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				new (require("./foo.js")).Foo();
			`,
			"/foo.js": `
				class Foo {}
				module.exports = {Foo};
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestCommonJSFromES6(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				const {foo} = require('./foo')
				console.log(foo(), bar())
				const {bar} = require('./bar') // This should not be hoisted
			`,
			"/foo.js": `
				export function foo() {
					return 'foo'
				}
			`,
			"/bar.js": `
				export function bar() {
					return 'bar'
				}
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestES6FromCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import {foo} from './foo'
				console.log(foo(), bar())
				import {bar} from './bar' // This should be hoisted
			`,
			"/foo.js": `
				exports.foo = function() {
					return 'foo'
				}
			`,
			"/bar.js": `
				exports.bar = function() {
					return 'bar'
				}
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}

// This test makes sure that ES6 imports are still recognized in nested
// scopes. It guards against bugs where require() calls are only recognized in
// the top-level module scope.
func TestNestedES6FromCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import {fn} from './foo'
				(() => {
					console.log(fn())
				})()
			`,
			"/foo.js": `
				exports.fn = function() {
					return 123
				}
			`,
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}



func TestExportFormsCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				require('./commonjs')
				require('./c')
				require('./d')
				require('./e')
				require('./f')
				require('./g')
				require('./h')
			`,
			"/commonjs.js": `
				export default 123
				export var v = 234
				export let l = 234
				export const c = 234
				export {Class as C}
				export function Fn() {}
				export class Class {}
				export * from './a'
				export * as b from './b'
			`,
			"/a.js": "export const abc = undefined",
			"/b.js": "export const xyz = null",
			"/c.js": "export default class {}",
			"/d.js": "export default class Foo {} Foo.prop = 123",
			"/e.js": "export default function() {}",
			"/f.js": "export default function foo() {} foo.prop = 123",
			"/g.js": "export default async function() {}",
			"/h.js": "export default async function foo() {} foo.prop = 123",
		},
		entryPaths: []string{"/entry.js"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}


func TestJSXImportsCommonJS(t *testing.T) {
	default_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.jsx": `
				import {elem, frag} from './custom-react'
				console.log(<div/>, <>fragment</>)
			`,
			"/custom-react.js": `
				module.exports = {}
			`,
		},
		entryPaths: []string{"/entry.jsx"},
		options: config.Options{
			Mode: config.ModeBundle,
			JSX: config.JSXOptions{
				Factory:  config.DefineExpr{Parts: []string{"elem"}},
				Fragment: config.DefineExpr{Parts: []string{"frag"}},
			},
			AbsOutputFile: "/out.js",
		},
	})
}