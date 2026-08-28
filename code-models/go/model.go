// The Go code model.
//
// Third language, and the one that tests hardest whether the C# schema was
// accidentally about C#. Go disagrees with both of the others in ways that
// have to show in the schema rather than be flattened into it:
//
//   * THERE ARE NO CLASSES. A struct is not a class with the inheritance
//     removed; it has embedded fields, which is composition with promotion,
//     and a method set that depends on whether the receiver is a pointer.
//   * EXPORTEDNESS IS SPELLING. `Name` is exported and `name` is not. There
//     is no `public` keyword to report, so an `Accessibility` enum copied
//     from C# would have exactly two reachable values and one of them would
//     be a lie.
//   * STRUCT TAGS ARE NOT ATTRIBUTES. They are a raw string on a FIELD,
//     conventionally holding key:"value" pairs that every library parses for
//     itself. They carry no type, take no arguments, and cannot appear on a
//     type or a method. `json:"sku,omitempty"` is the single most important
//     fact about a Go field for a generator emitting a wire shape, and it
//     fits nowhere in an attribute model.
//   * INTERFACES ARE SATISFIED SILENTLY. Like TypeScript, nothing declares
//     it — but unlike TypeScript, the method set is the whole test.

package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"path/filepath"
	"sort"
	"strings"

	"golang.org/x/tools/go/packages"
)

type Span struct {
	File        string `json:"file"`
	StartLine   int    `json:"startLine"`
	StartColumn int    `json:"startColumn"`
	EndLine     int    `json:"endLine"`
	EndColumn   int    `json:"endColumn"`
}

type Unresolved struct {
	Message string `json:"message"`
	At      *Span  `json:"at"`
}

// StructTag is Go's own metadata shape. Both the raw text and the parsed
// pairs are given: the raw is the truth, and the pairs are the convenience
// every caller would otherwise re-implement — badly, because the format has
// escaping rules people forget.
type StructTag struct {
	Raw   string     `json:"raw"`
	Pairs []TagValue `json:"pairs"`
}

type TagValue struct {
	Key     string   `json:"key"`
	Value   string   `json:"value"`
	Options []string `json:"options"`
}

type TypeRef struct {
	Text       string   `json:"text"`
	Kind       string   `json:"kind"`
	IsPointer  bool     `json:"isPointer"`
	IsSlice    bool     `json:"isSlice"`
	IsMap      bool     `json:"isMap"`
	Elem       *TypeRef `json:"elem"`
	Key        *TypeRef `json:"key"`
	IsExported bool     `json:"isExported"`
}

type Field struct {
	Name string `json:"name"`
	Type TypeRef `json:"type"`
	// Capitalisation, not a keyword.
	IsExported bool       `json:"isExported"`
	IsEmbedded bool       `json:"isEmbedded"`
	Tag        *StructTag `json:"tag"`
	Declaration *Span     `json:"declaration"`
}

type Param struct {
	Name string  `json:"name"`
	Type TypeRef `json:"type"`
}

type Method struct {
	Name string `json:"name"`
	// Whether the receiver is a pointer, which decides the method set — a
	// value of the type does not have pointer-receiver methods, so an
	// interface check that ignores this is wrong.
	PointerReceiver bool    `json:"pointerReceiver"`
	Params          []Param `json:"params"`
	Results         []Param `json:"results"`
	IsExported      bool    `json:"isExported"`
	Declaration     *Span   `json:"declaration"`
}

type Decl struct {
	Typename    string   `json:"__typename"`
	Name        string   `json:"name"`
	Package     string   `json:"package"`
	IsExported  bool     `json:"isExported"`
	Declarations []Span  `json:"declarations"`
	Methods     []Method `json:"methods"`
	// StructDecl
	Fields []Field `json:"fields"`
	// InterfaceDecl
	MethodNames []string `json:"methodNames"`
	// NamedDecl (a defined type over an existing one: `type Sku string`)
	Underlying *TypeRef `json:"underlying"`

	obj types.Object
}

type Model struct {
	Root       string
	Pkgs       []*packages.Package
	fset       *token.FileSet
	decls      []Decl
	unresolved []Unresolved
}

func Load(root string) (*Model, error) {
	abs, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}
	cfg := &packages.Config{
		Mode: packages.NeedName | packages.NeedFiles | packages.NeedSyntax |
			packages.NeedTypes | packages.NeedTypesInfo | packages.NeedDeps,
		Dir: abs,
	}
	pkgs, err := packages.Load(cfg, "./...")
	if err != nil {
		return nil, err
	}
	m := &Model{Root: abs, Pkgs: pkgs, fset: token.NewFileSet()}
	for _, p := range pkgs {
		if p.Fset != nil {
			m.fset = p.Fset
		}
		for _, e := range p.Errors {
			// Only what makes a SYMBOL wrong. A package that fails to build
			// for unrelated reasons still has a readable model, and refusing
			// to answer would be less useful than answering with the gaps
			// named.
			if strings.Contains(e.Msg, "undefined") || strings.Contains(e.Msg, "could not import") {
				m.unresolved = append(m.unresolved, Unresolved{Message: e.Msg})
			}
		}
	}
	m.build()
	return m, nil
}

func (m *Model) FileCount() int {
	n := 0
	for _, p := range m.Pkgs {
		n += len(p.GoFiles)
	}
	return n
}

func (m *Model) Unresolved() []Unresolved {
	if m.unresolved == nil {
		return []Unresolved{}
	}
	return m.unresolved
}

func (m *Model) build() {
	for _, p := range m.Pkgs {
		if p.Types == nil {
			continue
		}
		scope := p.Types.Scope()
		for _, name := range scope.Names() {
			obj := scope.Lookup(name)
			tn, ok := obj.(*types.TypeName)
			if !ok {
				continue
			}
			d := Decl{
				Name:       tn.Name(),
				Package:    p.PkgPath,
				IsExported: tn.Exported(),
				Declarations: []Span{m.span(obj.Pos(), obj.Pos())},
				Methods:    m.methodsOf(tn),
				obj:        obj,
			}
			switch u := tn.Type().Underlying().(type) {
			case *types.Struct:
				d.Typename = "StructDecl"
				d.Fields = m.fieldsOf(u)
			case *types.Interface:
				d.Typename = "InterfaceDecl"
				for i := 0; i < u.NumMethods(); i++ {
					d.MethodNames = append(d.MethodNames, u.Method(i).Name())
				}
			default:
				// `type Sku string` — a DEFINED type, which is not an alias
				// and not a struct. Go's third declaration kind, and one
				// neither C# nor TypeScript has in this shape.
				d.Typename = "NamedDecl"
				ref := m.typeRef(u)
				d.Underlying = &ref
			}
			m.decls = append(m.decls, d)
		}
	}
	sort.Slice(m.decls, func(i, j int) bool { return m.decls[i].Name < m.decls[j].Name })
}

func (m *Model) fieldsOf(s *types.Struct) []Field {
	out := []Field{}
	for i := 0; i < s.NumFields(); i++ {
		f := s.Field(i)
		var tag *StructTag
		if raw := s.Tag(i); raw != "" {
			t := parseTag(raw)
			tag = &t
		}
		out = append(out, Field{
			Name:        f.Name(),
			Type:        m.typeRef(f.Type()),
			IsExported:  f.Exported(),
			IsEmbedded:  f.Embedded(),
			Tag:         tag,
			Declaration: ptrSpan(m.span(f.Pos(), f.Pos())),
		})
	}
	return out
}

func (m *Model) methodsOf(tn *types.TypeName) []Method {
	out := []Method{}
	seen := map[string]bool{}
	for _, t := range []types.Type{tn.Type(), types.NewPointer(tn.Type())} {
		ms := types.NewMethodSet(t)
		for i := 0; i < ms.Len(); i++ {
			sel := ms.At(i)
			fn, ok := sel.Obj().(*types.Func)
			if !ok || seen[fn.Name()] {
				continue
			}
			seen[fn.Name()] = true
			sig, _ := fn.Type().(*types.Signature)
			if sig == nil {
				continue
			}
			_, isPtr := sig.Recv().Type().(*types.Pointer)
			out = append(out, Method{
				Name:            fn.Name(),
				PointerReceiver: isPtr,
				Params:          m.tuple(sig.Params()),
				Results:         m.tuple(sig.Results()),
				IsExported:      fn.Exported(),
				Declaration:     ptrSpan(m.span(fn.Pos(), fn.Pos())),
			})
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Name < out[j].Name })
	return out
}

func (m *Model) tuple(t *types.Tuple) []Param {
	out := []Param{}
	for i := 0; i < t.Len(); i++ {
		v := t.At(i)
		out = append(out, Param{Name: v.Name(), Type: m.typeRef(v.Type())})
	}
	return out
}

func (m *Model) typeRef(t types.Type) TypeRef {
	ref := TypeRef{Text: types.TypeString(t, relativeTo), Kind: "other"}
	switch v := t.(type) {
	case *types.Pointer:
		ref.IsPointer = true
		ref.Kind = "pointer"
		e := m.typeRef(v.Elem())
		ref.Elem = &e
	case *types.Slice:
		ref.IsSlice = true
		ref.Kind = "slice"
		e := m.typeRef(v.Elem())
		ref.Elem = &e
	case *types.Map:
		ref.IsMap = true
		ref.Kind = "map"
		e := m.typeRef(v.Elem())
		k := m.typeRef(v.Key())
		ref.Elem, ref.Key = &e, &k
	case *types.Named:
		ref.Kind = "named"
		ref.IsExported = v.Obj().Exported()
	case *types.Basic:
		ref.Kind = "basic"
	case *types.Interface:
		ref.Kind = "interface"
	}
	return ref
}

func relativeTo(p *types.Package) string { return p.Name() }

// Implementors answers the question Go shares with TypeScript and neither
// shares with C#: who satisfies this, given that nobody says so.
//
// The method set is the whole test, and it differs between a value and a
// pointer — so both are checked and which one satisfied is reported, because
// `var _ Stringer = Thing{}` and `var _ Stringer = &Thing{}` are different
// programs and a generator emitting one when it needed the other is broken.
func (m *Model) Implementors(name string) []map[string]any {
	var iface *types.Interface
	for _, d := range m.decls {
		if d.Name == name && d.Typename == "InterfaceDecl" {
			iface, _ = d.obj.Type().Underlying().(*types.Interface)
		}
	}
	out := []map[string]any{}
	if iface == nil {
		return out
	}
	for _, d := range m.decls {
		if d.obj == nil || d.Name == name {
			continue
		}
		t := d.obj.Type()
		byValue := types.Implements(t, iface)
		byPointer := types.Implements(types.NewPointer(t), iface)
		if byValue || byPointer {
			out = append(out, map[string]any{
				"decl": d, "byValue": byValue, "byPointer": byPointer,
			})
		}
	}
	return out
}

func (m *Model) Decls() []Decl { return m.decls }

func (m *Model) span(start, end token.Pos) Span {
	p := m.fset.Position(start)
	q := m.fset.Position(end)
	rel, err := filepath.Rel(m.Root, p.Filename)
	if err != nil {
		rel = p.Filename
	}
	return Span{File: rel, StartLine: p.Line, StartColumn: p.Column, EndLine: q.Line, EndColumn: q.Column}
}

func ptrSpan(s Span) *Span { return &s }

// parseTag reads Go's struct-tag convention: space-separated `key:"value"`,
// where the value is comma-separated and the first item is the name.
//
// Parsed here rather than left to the caller because every library that reads
// tags re-implements this, and the escaping rules are easy to get wrong.
func parseTag(raw string) StructTag {
	tag := StructTag{Raw: raw, Pairs: []TagValue{}}
	rest := raw
	for rest != "" {
		i := strings.IndexByte(rest, ':')
		if i < 0 {
			break
		}
		key := strings.TrimSpace(rest[:i])
		rest = rest[i+1:]
		if rest == "" || rest[0] != '"' {
			break
		}
		j := strings.IndexByte(rest[1:], '"')
		if j < 0 {
			break
		}
		value := rest[1 : j+1]
		rest = strings.TrimSpace(rest[j+2:])
		parts := strings.Split(value, ",")
		tag.Pairs = append(tag.Pairs, TagValue{
			Key: key, Value: parts[0], Options: parts[1:],
		})
	}
	return tag
}

var _ = ast.Print
var _ = fmt.Sprintf

// ---------------------------------------------------------------------------
// References
// ---------------------------------------------------------------------------
//
// Added after the first pass claimed a code model has no business answering
// "where is this used". That was wrong, and wrong in a way worth recording: a
// generator exists to make a meta-pattern official ("the API layer holds no
// business validation"), and a meta-pattern worth stating almost always has
// exceptions worth stating too — "except for fields the database layer
// reads". An exception phrased in terms of USE SITES is a references query,
// and a generator that cannot ask one has to be told the exceptions by hand,
// which is the list nobody maintains.
//
// In Go this costs nothing. `packages.NeedTypesInfo` already fills
// `Info.Uses`, an identifier-to-object map for every file loaded — a
// references index, computed before anyone asked. Grouping it is the whole
// implementation.

// Reference is one use of a symbol, and — the part that matters — WHO used it.
//
// A bare location cannot answer "is this referenced by the database layer".
// The referring declaration and its package can, which is why they are here
// and not left to the caller to reconstruct from a line number.
type Reference struct {
	Span            Span   `json:"span"`
	FromPackage     string `json:"fromPackage"`
	FromDeclaration string `json:"fromDeclaration"`
	IsWrite         bool   `json:"isWrite"`
}

// References returns every use of `symbol`, named `Type`, `Type.Member`, or a
// bare function name.
func (m *Model) References(symbol string) []Reference {
	target := m.lookup(symbol)
	if target == nil {
		return []Reference{}
	}
	out := []Reference{}
	for _, p := range m.Pkgs {
		if p.TypesInfo == nil {
			continue
		}
		for ident, obj := range p.TypesInfo.Uses {
			if obj != target {
				continue
			}
			out = append(out, Reference{
				Span:            m.span(ident.Pos(), ident.End()),
				FromPackage:     p.PkgPath,
				FromDeclaration: m.enclosing(p, ident.Pos()),
				IsWrite:         m.isWrite(p, ident),
			})
		}
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Span.File != out[j].Span.File {
			return out[i].Span.File < out[j].Span.File
		}
		return out[i].Span.StartLine < out[j].Span.StartLine
	})
	return out
}

// lookup resolves `Type`, `Type.Member` or a bare name to an object.
func (m *Model) lookup(symbol string) types.Object {
	typeName, member, hasMember := strings.Cut(symbol, ".")
	for _, p := range m.Pkgs {
		if p.Types == nil {
			continue
		}
		scope := p.Types.Scope()
		obj := scope.Lookup(typeName)
		if obj == nil {
			continue
		}
		if !hasMember {
			return obj
		}
		named, ok := obj.Type().(*types.Named)
		if !ok {
			continue
		}
		for i := 0; i < named.NumMethods(); i++ {
			if named.Method(i).Name() == member {
				return named.Method(i)
			}
		}
		if st, ok := named.Underlying().(*types.Struct); ok {
			for i := 0; i < st.NumFields(); i++ {
				if st.Field(i).Name() == member {
					return st.Field(i)
				}
			}
		}
	}
	return nil
}

// enclosing names the declaration a position sits inside, which is the thing
// a rule about layers actually tests.
func (m *Model) enclosing(p *packages.Package, pos token.Pos) string {
	for _, file := range p.Syntax {
		if pos < file.Pos() || pos > file.End() {
			continue
		}
		for _, d := range file.Decls {
			if pos < d.Pos() || pos > d.End() {
				continue
			}
			switch decl := d.(type) {
			case *ast.FuncDecl:
				if decl.Recv != nil && len(decl.Recv.List) > 0 {
					return receiverName(decl.Recv.List[0].Type) + "." + decl.Name.Name
				}
				return decl.Name.Name
			case *ast.GenDecl:
				for _, spec := range decl.Specs {
					if ts, ok := spec.(*ast.TypeSpec); ok {
						return ts.Name.Name
					}
				}
			}
		}
	}
	return ""
}

func receiverName(expr ast.Expr) string {
	switch t := expr.(type) {
	case *ast.StarExpr:
		return receiverName(t.X)
	case *ast.Ident:
		return t.Name
	case *ast.IndexExpr:
		return receiverName(t.X)
	}
	return ""
}

// isWrite: whether this use is the left side of an assignment. A rule like
// "nothing outside the domain may set this" needs reads and writes told
// apart, and a flat symbol graph cannot.
func (m *Model) isWrite(p *packages.Package, ident *ast.Ident) bool {
	for _, file := range p.Syntax {
		if ident.Pos() < file.Pos() || ident.Pos() > file.End() {
			continue
		}
		found := false
		ast.Inspect(file, func(n ast.Node) bool {
			assign, ok := n.(*ast.AssignStmt)
			if !ok {
				return true
			}
			for _, lhs := range assign.Lhs {
				if lhs.Pos() <= ident.Pos() && ident.End() <= lhs.End() {
					found = true
				}
			}
			return true
		})
		return found
	}
	return false
}
