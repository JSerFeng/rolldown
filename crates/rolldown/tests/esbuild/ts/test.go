package bundler_tests

import (
	"testing"

	"github.com/evanw/esbuild/internal/compat"
	"github.com/evanw/esbuild/internal/config"
)

var ts_suite = suite{
	name: "ts",
}

func TestTypeScriptDecorators(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.js": `
				import all from './all'
				import all_computed from './all_computed'
				import {a} from './a'
				import {b} from './b'
				import {c} from './c'
				import {d} from './d'
				import e from './e'
				import f from './f'
				import g from './g'
				import h from './h'
				import {i} from './i'
				import {j} from './j'
				import k from './k'
				import {fn} from './arguments'
				console.log(all, all_computed, a, b, c, d, e, f, g, h, i, j, k, fn)
			`,
			"/all.ts": `
				@x.y()
				@new y.x()
				export default class Foo {
					@x @y mUndef
					@x @y mDef = 1
					@x @y method(@x0 @y0 arg0, @x1 @y1 arg1) { return new Foo }
					@x @y declare mDecl
					constructor(@x0 @y0 arg0, @x1 @y1 arg1) {}

					@x @y static sUndef
					@x @y static sDef = new Foo
					@x @y static sMethod(@x0 @y0 arg0, @x1 @y1 arg1) { return new Foo }
					@x @y static declare mDecl
				}
			`,
			"/all_computed.ts": `
				@x?.[_ + 'y']()
				@new y?.[_ + 'x']()
				export default class Foo {
					@x @y [mUndef()]
					@x @y [mDef()] = 1
					@x @y [method()](@x0 @y0 arg0, @x1 @y1 arg1) { return new Foo }
					@x @y declare [mDecl()]

					// Side effect order must be preserved even for fields without decorators
					[xUndef()]
					[xDef()] = 2
					static [yUndef()]
					static [yDef()] = 3

					@x @y static [sUndef()]
					@x @y static [sDef()] = new Foo
					@x @y static [sMethod()](@x0 @y0 arg0, @x1 @y1 arg1) { return new Foo }
					@x @y static declare [mDecl()]
				}
			`,
			"/a.ts": `
				@x(() => 0) @y(() => 1)
				class a_class {
					fn() { return new a_class }
					static z = new a_class
				}
				export let a = a_class
			`,
			"/b.ts": `
				@x(() => 0) @y(() => 1)
				abstract class b_class {
					fn() { return new b_class }
					static z = new b_class
				}
				export let b = b_class
			`,
			"/c.ts": `
				@x(() => 0) @y(() => 1)
				export class c {
					fn() { return new c }
					static z = new c
				}
			`,
			"/d.ts": `
				@x(() => 0) @y(() => 1)
				export abstract class d {
					fn() { return new d }
					static z = new d
				}
			`,
			"/e.ts": `
				@x(() => 0) @y(() => 1)
				export default class {}
			`,
			"/f.ts": `
				@x(() => 0) @y(() => 1)
				export default class f {
					fn() { return new f }
					static z = new f
				}
			`,
			"/g.ts": `
				@x(() => 0) @y(() => 1)
				export default abstract class {}
			`,
			"/h.ts": `
				@x(() => 0) @y(() => 1)
				export default abstract class h {
					fn() { return new h }
					static z = new h
				}
			`,
			"/i.ts": `
				class i_class {
					@x(() => 0) @y(() => 1)
					foo
				}
				export let i = i_class
			`,
			"/j.ts": `
				export class j {
					@x(() => 0) @y(() => 1)
					foo() {}
				}
			`,
			"/k.ts": `
				export default class {
					foo(@x(() => 0) @y(() => 1) x) {}
				}
			`,
			"/arguments.ts": `
				function dec(x: any): any {}
				export function fn(x: string): any {
					class Foo {
						@dec(arguments[0])
						[arguments[0]]() {}
					}
					return Foo;
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


// See: https://github.com/evanw/esbuild/issues/2147
func Test_type_script_decorator_scope_issue2147(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				let foo = 1
				class Foo {
					method1(@dec(foo) foo = 2) {}
					method2(@dec(() => foo) foo = 3) {}
				}

				class Bar {
					static x = class {
						static y = () => {
							let bar = 1
							@dec(bar)
							@dec(() => bar)
							class Baz {
								@dec(bar) method1() {}
								@dec(() => bar) method2() {}
								method3(@dec(() => bar) bar) {}
								method4(@dec(() => bar) bar) {}
							}
							return Baz
						}
					}
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModePassThrough,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestTSExportDefaultTypeIssue316(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				import dc_def, { bar as dc } from './keep/declare-class'
				import dl_def, { bar as dl } from './keep/declare-let'
				import im_def, { bar as im } from './keep/interface-merged'
				import in_def, { bar as _in } from './keep/interface-nested'
				import tn_def, { bar as tn } from './keep/type-nested'
				import vn_def, { bar as vn } from './keep/value-namespace'
				import vnm_def, { bar as vnm } from './keep/value-namespace-merged'

				import i_def, { bar as i } from './remove/interface'
				import ie_def, { bar as ie } from './remove/interface-exported'
				import t_def, { bar as t } from './remove/type'
				import te_def, { bar as te } from './remove/type-exported'
				import ton_def, { bar as ton } from './remove/type-only-namespace'
				import tone_def, { bar as tone } from './remove/type-only-namespace-exported'

				export default [
					dc_def, dc,
					dl_def, dl,
					im_def, im,
					in_def, _in,
					tn_def, tn,
					vn_def, vn,
					vnm_def, vnm,

					i,
					ie,
					t,
					te,
					ton,
					tone,
				]
			`,
			"/keep/declare-class.ts": `
				declare class foo {}
				export default foo
				export let bar = 123
			`,
			"/keep/declare-let.ts": `
				declare let foo: number
				export default foo
				export let bar = 123
			`,
			"/keep/interface-merged.ts": `
				class foo {
					static x = new foo
				}
				interface foo {}
				export default foo
				export let bar = 123
			`,
			"/keep/interface-nested.ts": `
				if (true) {
					interface foo {}
				}
				export default foo
				export let bar = 123
			`,
			"/keep/type-nested.ts": `
				if (true) {
					type foo = number
				}
				export default foo
				export let bar = 123
			`,
			"/keep/value-namespace.ts": `
				namespace foo {
					export let num = 0
				}
				export default foo
				export let bar = 123
			`,
			"/keep/value-namespace-merged.ts": `
				namespace foo {
					export type num = number
				}
				namespace foo {
					export let num = 0
				}
				export default foo
				export let bar = 123
			`,
			"/remove/interface.ts": `
				interface foo { }
				export default foo
				export let bar = 123
			`,
			"/remove/interface-exported.ts": `
				export interface foo { }
				export default foo
				export let bar = 123
			`,
			"/remove/type.ts": `
				type foo = number
				export default foo
				export let bar = 123
			`,
			"/remove/type-exported.ts": `
				export type foo = number
				export default foo
				export let bar = 123
			`,
			"/remove/type-only-namespace.ts": `
				namespace foo {
					export type num = number
				}
				export default foo
				export let bar = 123
			`,
			"/remove/type-only-namespace-exported.ts": `
				export namespace foo {
					export type num = number
				}
				export default foo
				export let bar = 123
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
	})
}


func TestTSImplicitExtensionsMissing(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				import './mjs.mjs'
				import './cjs.cjs'
				import './js.js'
				import './jsx.jsx'
			`,
			"/mjs.ts":      ``,
			"/mjs.tsx":     ``,
			"/cjs.ts":      ``,
			"/cjs.tsx":     ``,
			"/js.ts.js":    ``,
			"/jsx.tsx.jsx": ``,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		expectedScanLog: `entry.ts: ERROR: Could not resolve "./mjs.mjs"
entry.ts: ERROR: Could not resolve "./cjs.cjs"
entry.ts: ERROR: Could not resolve "./js.js"
entry.ts: ERROR: Could not resolve "./jsx.jsx"
`,
	})
}

func TestThisInsideFunctionTS(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				function foo(x = this) { console.log(this) }
				const objFoo = {
					foo(x = this) { console.log(this) }
				}
				class Foo {
					x = this
					static y = this.z
					foo(x = this) { console.log(this) }
					static bar(x = this) { console.log(this) }
				}
				new Foo(foo(objFoo))
				if (nested) {
					function bar(x = this) { console.log(this) }
					const objBar = {
						foo(x = this) { console.log(this) }
					}
					class Bar {
						x = this
						static y = this.z
						foo(x = this) { console.log(this) }
						static bar(x = this) { console.log(this) }
					}
					new Bar(bar(objBar))
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModeBundle,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.False,
		},
	})
}

func TestThisInsideFunctionTSUseDefineForClassFields(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				function foo(x = this) { console.log(this) }
				const objFoo = {
					foo(x = this) { console.log(this) }
				}
				class Foo {
					x = this
					static y = this.z
					foo(x = this) { console.log(this) }
					static bar(x = this) { console.log(this) }
				}
				new Foo(foo(objFoo))
				if (nested) {
					function bar(x = this) { console.log(this) }
					const objBar = {
						foo(x = this) { console.log(this) }
					}
					class Bar {
						x = this
						static y = this.z
						foo(x = this) { console.log(this) }
						static bar(x = this) { console.log(this) }
					}
					new Bar(bar(objBar))
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModeBundle,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.True,
		},
	})
}

func TestThisInsideFunctionTSNoBundle(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				function foo(x = this) { console.log(this) }
				const objFoo = {
					foo(x = this) { console.log(this) }
				}
				class Foo {
					x = this
					static y = this.z
					foo(x = this) { console.log(this) }
					static bar(x = this) { console.log(this) }
				}
				new Foo(foo(objFoo))
				if (nested) {
					function bar(x = this) { console.log(this) }
					const objBar = {
						foo(x = this) { console.log(this) }
					}
					class Bar {
						x = this
						static y = this.z
						foo(x = this) { console.log(this) }
						static bar(x = this) { console.log(this) }
					}
					new Bar(bar(objBar))
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModePassThrough,
			AbsOutputFile: "/out.js",
		},
	})
}

func TestThisInsideFunctionTSNoBundleUseDefineForClassFields(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				function foo(x = this) { console.log(this) }
				const objFoo = {
					foo(x = this) { console.log(this) }
				}
				class Foo {
					x = this
					static y = this.z
					foo(x = this) { console.log(this) }
					static bar(x = this) { console.log(this) }
				}
				new Foo(foo(objFoo))
				if (nested) {
					function bar(x = this) { console.log(this) }
					const objBar = {
						foo(x = this) { console.log(this) }
					}
					class Bar {
						x = this
						static y = this.z
						foo(x = this) { console.log(this) }
						static bar(x = this) { console.log(this) }
					}
					new Bar(bar(objBar))
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModePassThrough,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.True,
		},
	})
}


func TestTSComputedClassFieldUseDefineTrueLower(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				class Foo {
					[q];
					[r] = s;
					@dec
					[x];
					@dec
					[y] = z;
				}
				new Foo()
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModePassThrough,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.True,
			UnsupportedJSFeatures:   compat.ClassField,
		},
	})
}

func TestTSAbstractClassFieldUseAssign(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				const keepThis = Symbol('keepThis')
				declare const AND_REMOVE_THIS: unique symbol
				abstract class Foo {
					REMOVE_THIS: any
					[keepThis]: any
					abstract REMOVE_THIS_TOO: any
					abstract [AND_REMOVE_THIS]: any
					abstract [(x => y => x + y)('nested')('scopes')]: any
				}
				(() => new Foo())()
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModePassThrough,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.False,
		},
	})
}

func TestTSAbstractClassFieldUseDefine(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				const keepThisToo = Symbol('keepThisToo')
				declare const REMOVE_THIS_TOO: unique symbol
				abstract class Foo {
					keepThis: any
					[keepThisToo]: any
					abstract REMOVE_THIS: any
					abstract [REMOVE_THIS_TOO]: any
					abstract [(x => y => x + y)('nested')('scopes')]: any
				}
				(() => new Foo())()
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:                    config.ModePassThrough,
			AbsOutputFile:           "/out.js",
			UseDefineForClassFields: config.True,
		},
	})
}


func TestTSImportCTS(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				require('./required.cjs')
			`,
			"/required.cjs": `
				console.log('works')
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
			OutputFormat:  config.FormatCommonJS,
		},
	})
}

func TestTSSideEffectsFalseWarningTypeDeclarations(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				import "some-js"
				import "some-ts"
				import "empty-js"
				import "empty-ts"
				import "empty-dts"
			`,

			"/node_modules/some-js/package.json": `{ "main": "./foo.js", "sideEffects": false }`,
			"/node_modules/some-js/foo.js":       `console.log('foo')`,

			"/node_modules/some-ts/package.json": `{ "main": "./foo.ts", "sideEffects": false }`,
			"/node_modules/some-ts/foo.ts":       `console.log('foo' as string)`,

			"/node_modules/empty-js/package.json": `{ "main": "./foo.js", "sideEffects": false }`,
			"/node_modules/empty-js/foo.js":       ``,

			"/node_modules/empty-ts/package.json": `{ "main": "./foo.ts", "sideEffects": false }`,
			"/node_modules/empty-ts/foo.ts":       `export type Foo = number`,

			"/node_modules/empty-dts/package.json": `{ "main": "./foo.d.ts", "sideEffects": false }`,
			"/node_modules/empty-dts/foo.d.ts":     `export type Foo = number`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		expectedScanLog: `entry.ts: WARNING: Ignoring this import because "node_modules/some-js/foo.js" was marked as having no side effects
node_modules/some-js/package.json: NOTE: "sideEffects" is false in the enclosing "package.json" file
entry.ts: WARNING: Ignoring this import because "node_modules/some-ts/foo.ts" was marked as having no side effects
node_modules/some-ts/package.json: NOTE: "sideEffects" is false in the enclosing "package.json" file
`,
	})
}

func TestTSSiblingNamespace(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/let.ts": `
				export namespace x { export let y = 123 }
				export namespace x { export let z = y }
			`,
			"/function.ts": `
				export namespace x { export function y() {} }
				export namespace x { export let z = y }
			`,
			"/class.ts": `
				export namespace x { export class y {} }
				export namespace x { export let z = y }
			`,
			"/namespace.ts": `
				export namespace x { export namespace y { 0 } }
				export namespace x { export let z = y }
			`,
			"/enum.ts": `
				export namespace x { export enum y {} }
				export namespace x { export let z = y }
			`,
		},
		entryPaths: []string{
			"/let.ts",
			"/function.ts",
			"/class.ts",
			"/namespace.ts",
			"/enum.ts",
		},
		options: config.Options{
			Mode:         config.ModePassThrough,
			AbsOutputDir: "/out",
		},
	})
}

func TestTSSiblingEnum(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/number.ts": `
				export enum x { y, yy = y }
				export enum x { z = y + 1 }

				declare let y: any, z: any
				export namespace x { console.log(y, z) }
				console.log(x.y, x.z)
			`,
			"/string.ts": `
				export enum x { y = 'a', yy = y }
				export enum x { z = y }

				declare let y: any, z: any
				export namespace x { console.log(y, z) }
				console.log(x.y, x.z)
			`,
			"/propagation.ts": `
				export enum a { b = 100 }
				export enum x {
					c = a.b,
					d = c * 2,
					e = x.d ** 2,
					f = x['e'] / 4,
				}
				export enum x { g = f >> 4 }
				console.log(a.b, a['b'], x.g, x['g'])
			`,
			"/nested-number.ts": `
				export namespace foo { export enum x { y, yy = y } }
				export namespace foo { export enum x { z = y + 1 } }

				declare let y: any, z: any
				export namespace foo.x {
					console.log(y, z)
					console.log(x.y, x.z)
				}
			`,
			"/nested-string.ts": `
				export namespace foo { export enum x { y = 'a', yy = y } }
				export namespace foo { export enum x { z = y } }

				declare let y: any, z: any
				export namespace foo.x {
					console.log(y, z)
					console.log(x.y, x.z)
				}
			`,
			"/nested-propagation.ts": `
				export namespace n { export enum a { b = 100 } }
				export namespace n {
					export enum x {
						c = n.a.b,
						d = c * 2,
						e = x.d ** 2,
						f = x['e'] / 4,
					}
				}
				export namespace n {
					export enum x { g = f >> 4 }
					console.log(a.b, n.a.b, n['a']['b'], x.g, n.x.g, n['x']['g'])
				}
			`,
		},
		entryPaths: []string{
			"/number.ts",
			"/string.ts",
			"/propagation.ts",
			"/nested-number.ts",
			"/nested-string.ts",
			"/nested-propagation.ts",
		},
		options: config.Options{
			Mode:         config.ModePassThrough,
			AbsOutputDir: "/out",
		},
	})
}

func TestTSEnumDefine(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				enum a { b = 123, c = d }
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:         config.ModePassThrough,
			AbsOutputDir: "/out",
			Defines: &config.ProcessedDefines{
				IdentifierDefines: map[string]config.DefineData{
					"d": {
						DefineExpr: &config.DefineExpr{
							Parts: []string{"b"},
						},
					},
				},
			},
		},
	})
}



// This checks that we don't generate a warning for code that the TypeScript
// compiler generates that looks like this:
//
//	var __rest = (this && this.__rest) || function (s, e) {
//	  ...
//	};
func TestTSThisIsUndefinedWarning(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/warning1.ts": `export var foo = this`,
			"/warning2.ts": `export var foo = this || this.foo`,
			"/warning3.ts": `export var foo = this ? this.foo : null`,

			"/silent1.ts": `export var foo = this && this.foo`,
			"/silent2.ts": `export var foo = this && (() => this.foo)`,
		},
		entryPaths: []string{
			"/warning1.ts",
			"/warning2.ts",
			"/warning3.ts",

			"/silent1.ts",
			"/silent2.ts",
		},
		options: config.Options{
			Mode:         config.ModeBundle,
			AbsOutputDir: "/out",
		},
		debugLogs: true,
		expectedScanLog: `warning1.ts: DEBUG: Top-level "this" will be replaced with undefined since this file is an ECMAScript module
warning1.ts: NOTE: This file is considered to be an ECMAScript module because of the "export" keyword here:
warning2.ts: DEBUG: Top-level "this" will be replaced with undefined since this file is an ECMAScript module
warning2.ts: NOTE: This file is considered to be an ECMAScript module because of the "export" keyword here:
warning3.ts: DEBUG: Top-level "this" will be replaced with undefined since this file is an ECMAScript module
warning3.ts: NOTE: This file is considered to be an ECMAScript module because of the "export" keyword here:
`,
	})
}

func TestTSCommonJSVariableInESMTypeModule(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts":     `module.exports = null`,
			"/package.json": `{ "type": "module" }`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:          config.ModeBundle,
			AbsOutputFile: "/out.js",
		},
		expectedScanLog: `entry.ts: WARNING: The CommonJS "module" variable is treated as a global variable in an ECMAScript module and may not work as expected
package.json: NOTE: This file is considered to be an ECMAScript module because the enclosing "package.json" file sets the type of this file to "module":
NOTE: Node's package format requires that CommonJS files in a "type": "module" package use the ".cjs" file extension. If you are using TypeScript, you can use the ".cts" file extension with esbuild instead.
`,
	})
}

func TestEnumRulesFrom_TypeScript_5_0(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/supported.ts": `
				// From https://github.com/microsoft/TypeScript/pull/50528:
				// "An expression is considered a constant expression if it is
				const enum Foo {
					// a number or string literal,
					X0 = 123,
					X1 = 'x',

					// a unary +, -, or ~ applied to a numeric constant expression,
					X2 = +1,
					X3 = -2,
					X4 = ~3,

					// a binary +, -, *, /, %, **, <<, >>, >>>, |, &, ^ applied to two numeric constant expressions,
					X5 = 1 + 2,
					X6 = 1 - 2,
					X7 = 2 * 3,
					X8 = 1 / 2,
					X9 = 3 % 2,
					X10 = 2 ** 3,
					X11 = 1 << 2,
					X12 = -9 >> 1,
					X13 = -9 >>> 1,
					X14 = 5 | 12,
					X15 = 5 & 12,
					X16 = 5 ^ 12,

					// a binary + applied to two constant expressions whereof at least one is a string,
					X17 = 'x' + 0,
					X18 = 0 + 'x',
					X19 = 'x' + 'y',
					X20 = '' + NaN,
					X21 = '' + Infinity,
					X22 = '' + -Infinity,
					X23 = '' + -0,

					// a template expression where each substitution expression is a constant expression,
					X24 = ` + "`A${0}B${'x'}C${1 + 3 - 4 / 2 * 5 ** 6}D`" + `,

					// a parenthesized constant expression,
					X25 = (321),

					// a dotted name (e.g. x.y.z) that references a const variable with a constant expression initializer and no type annotation,
					/* (we don't implement this one) */

					// a dotted name that references an enum member with an enum literal type, or
					X26 = X0,
					X27 = X0 + 'x',
					X28 = 'x' + X0,
					X29 = ` + "`a${X0}b`" + `,
					X30 = Foo.X0,
					X31 = Foo.X0 + 'x',
					X32 = 'x' + Foo.X0,
					X33 = ` + "`a${Foo.X0}b`" + `,

					// a dotted name indexed by a string literal (e.g. x.y["z"]) that references an enum member with an enum literal type."
					X34 = X1,
					X35 = X1 + 'y',
					X36 = 'y' + X1,
					X37 = ` + "`a${X1}b`" + `,
					X38 = Foo['X1'],
					X39 = Foo['X1'] + 'y',
					X40 = 'y' + Foo['X1'],
					X41 = ` + "`a${Foo['X1']}b`" + `,
				}

				console.log(
					// a number or string literal,
					Foo.X0,
					Foo.X1,

					// a unary +, -, or ~ applied to a numeric constant expression,
					Foo.X2,
					Foo.X3,
					Foo.X4,

					// a binary +, -, *, /, %, **, <<, >>, >>>, |, &, ^ applied to two numeric constant expressions,
					Foo.X5,
					Foo.X6,
					Foo.X7,
					Foo.X8,
					Foo.X9,
					Foo.X10,
					Foo.X11,
					Foo.X12,
					Foo.X13,
					Foo.X14,
					Foo.X15,
					Foo.X16,

					// a template expression where each substitution expression is a constant expression,
					Foo.X17,
					Foo.X18,
					Foo.X19,
					Foo.X20,
					Foo.X21,
					Foo.X22,
					Foo.X23,

					// a template expression where each substitution expression is a constant expression,
					Foo.X24,

					// a parenthesized constant expression,
					Foo.X25,

					// a dotted name that references an enum member with an enum literal type, or
					Foo.X26,
					Foo.X27,
					Foo.X28,
					Foo.X29,
					Foo.X30,
					Foo.X31,
					Foo.X32,
					Foo.X33,

					// a dotted name indexed by a string literal (e.g. x.y["z"]) that references an enum member with an enum literal type."
					Foo.X34,
					Foo.X35,
					Foo.X36,
					Foo.X37,
					Foo.X38,
					Foo.X39,
					Foo.X40,
					Foo.X41,
				)
			`,
			"/not-supported.ts": `
				const enum NonIntegerNumberToString {
					SUPPORTED = '' + 1,
					UNSUPPORTED = '' + 1.5,
				}
				console.log(
					NonIntegerNumberToString.SUPPORTED,
					NonIntegerNumberToString.UNSUPPORTED,
				)

				const enum OutOfBoundsNumberToString {
					SUPPORTED = '' + 1_000_000_000,
					UNSUPPORTED = '' + 1_000_000_000_000,
				}
				console.log(
					OutOfBoundsNumberToString.SUPPORTED,
					OutOfBoundsNumberToString.UNSUPPORTED,
				)

				const enum TemplateExpressions {
					// TypeScript enums don't handle any of these
					NULL = '' + null,
					TRUE = '' + true,
					FALSE = '' + false,
					BIGINT = '' + 123n,
				}
				console.log(
					TemplateExpressions.NULL,
					TemplateExpressions.TRUE,
					TemplateExpressions.FALSE,
					TemplateExpressions.BIGINT,
				)
			`,
		},
		entryPaths: []string{
			"/supported.ts",
			"/not-supported.ts",
		},
		options: config.Options{
			Mode:         config.ModeBundle,
			AbsOutputDir: "/out",
		},
	})
}

