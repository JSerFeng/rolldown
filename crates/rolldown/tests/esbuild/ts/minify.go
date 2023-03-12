
func TestTSMinifyEnum(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				enum Foo { A, B, C = Foo }
			`,
			"/b.ts": `
				export enum Foo { X, Y, Z = Foo }
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:      true,
			MinifyWhitespace:  true,
			MinifyIdentifiers: true,
			AbsOutputDir:      "/",
		},
	})
}

func TestTSMinifyNestedEnum(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				function foo() { enum Foo { A, B, C = Foo } return Foo }
			`,
			"/b.ts": `
				export function foo() { enum Foo { X, Y, Z = Foo } return Foo }
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:      true,
			MinifyWhitespace:  true,
			MinifyIdentifiers: true,
			AbsOutputDir:      "/",
		},
	})
}

func TestTSMinifyNestedEnumNoLogicalAssignment(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				function foo() { enum Foo { A, B, C = Foo } return Foo }
			`,
			"/b.ts": `
				export function foo() { enum Foo { X, Y, Z = Foo } return Foo }
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:          true,
			MinifyWhitespace:      true,
			MinifyIdentifiers:     true,
			AbsOutputDir:          "/",
			UnsupportedJSFeatures: compat.LogicalAssignment,
		},
	})
}

func TestTSMinifyNestedEnumNoArrow(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				function foo() { enum Foo { A, B, C = Foo } return Foo }
			`,
			"/b.ts": `
				export function foo() { enum Foo { X, Y, Z = Foo } return Foo }
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:          true,
			MinifyWhitespace:      true,
			MinifyIdentifiers:     true,
			AbsOutputDir:          "/",
			UnsupportedJSFeatures: compat.Arrow,
		},
	})
}

func TestTSMinifyNamespace(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
			"/b.ts": `
				export namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:      true,
			MinifyWhitespace:  true,
			MinifyIdentifiers: true,
			AbsOutputDir:      "/",
		},
	})
}

func TestTSMinifyNamespaceNoLogicalAssignment(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
			"/b.ts": `
				export namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:          true,
			MinifyWhitespace:      true,
			MinifyIdentifiers:     true,
			AbsOutputDir:          "/",
			UnsupportedJSFeatures: compat.LogicalAssignment,
		},
	})
}

func TestTSMinifyNamespaceNoArrow(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/a.ts": `
				namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
			"/b.ts": `
				export namespace Foo {
					export namespace Bar {
						foo(Foo, Bar)
					}
				}
			`,
		},
		entryPaths: []string{"/a.ts", "/b.ts"},
		options: config.Options{
			MinifySyntax:          true,
			MinifyWhitespace:      true,
			MinifyIdentifiers:     true,
			AbsOutputDir:          "/",
			UnsupportedJSFeatures: compat.Arrow,
		},
	})
}

func TestTSMinifyDerivedClass(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				class Foo extends Bar {
					foo = 1;
					bar = 2;
					constructor() {
						super();
						foo();
						bar();
					}
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			MinifySyntax:          true,
			UnsupportedJSFeatures: es(2015),
			AbsOutputFile:         "/out.js",
		},
	})
}


func TestTSMinifiedBundleES6(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				import {foo} from './a'
				console.log(foo())
			`,
			"/a.ts": `
				export function foo() {
					return 123
				}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:              config.ModeBundle,
			MinifySyntax:      true,
			MinifyWhitespace:  true,
			MinifyIdentifiers: true,
			AbsOutputFile:     "/out.js",
		},
	})
}

func TestTSMinifiedBundleCommonJS(t *testing.T) {
	ts_suite.expectBundled(t, bundled{
		files: map[string]string{
			"/entry.ts": `
				const {foo} = require('./a')
				console.log(foo(), require('./j.json'))
			`,
			"/a.ts": `
				exports.foo = function() {
					return 123
				}
			`,
			"/j.json": `
				{"test": true}
			`,
		},
		entryPaths: []string{"/entry.ts"},
		options: config.Options{
			Mode:              config.ModeBundle,
			MinifySyntax:      true,
			MinifyWhitespace:  true,
			MinifyIdentifiers: true,
			AbsOutputFile:     "/out.js",
		},
	})
}