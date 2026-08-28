// Go's schema. Nothing here is shared with C#'s or TypeScript's except the
// three core types, which is the finding rather than an accident.

package main

import "github.com/graphql-go/graphql"

func nn(t graphql.Type) graphql.Type      { return graphql.NewNonNull(t) }
func nnList(t graphql.Type) graphql.Type  { return graphql.NewNonNull(graphql.NewList(graphql.NewNonNull(t))) }

func buildSchema(model *Model) (graphql.Schema, error) {
	span := graphql.NewObject(graphql.ObjectConfig{
		Name: "Span",
		Fields: graphql.Fields{
			"file":        &graphql.Field{Type: nn(graphql.String)},
			"startLine":   &graphql.Field{Type: nn(graphql.Int)},
			"startColumn": &graphql.Field{Type: nn(graphql.Int)},
			"endLine":     &graphql.Field{Type: nn(graphql.Int)},
			"endColumn":   &graphql.Field{Type: nn(graphql.Int)},
		},
	})

	serverInfo := graphql.NewObject(graphql.ObjectConfig{
		Name: "ServerInfo",
		Fields: graphql.Fields{
			"name":      &graphql.Field{Type: nn(graphql.String)},
			"version":   &graphql.Field{Type: nn(graphql.String)},
			"language":  &graphql.Field{Type: nn(graphql.String)},
			"root":      &graphql.Field{Type: nn(graphql.String)},
			"fileCount": &graphql.Field{Type: nn(graphql.Int)},
		},
	})

	unresolved := graphql.NewObject(graphql.ObjectConfig{
		Name: "UnresolvedReference",
		Description: "Something the model could not bind. Reported rather than " +
			"swallowed: a generator that ran against a half-resolved package and " +
			"believed it saw everything emits confidently wrong code.",
		Fields: graphql.Fields{
			"message": &graphql.Field{Type: nn(graphql.String)},
			"at":      &graphql.Field{Type: span},
		},
	})

	var typeRef *graphql.Object
	typeRef = graphql.NewObject(graphql.ObjectConfig{
		Name: "TypeRef",
		Fields: graphql.FieldsThunk(func() graphql.Fields {
			return graphql.Fields{
				"text":       &graphql.Field{Type: nn(graphql.String)},
				"kind":       &graphql.Field{Type: nn(graphql.String)},
				"isPointer":  &graphql.Field{Type: nn(graphql.Boolean)},
				"isSlice":    &graphql.Field{Type: nn(graphql.Boolean)},
				"isMap":      &graphql.Field{Type: nn(graphql.Boolean)},
				"elem":       &graphql.Field{Type: typeRef},
				"key":        &graphql.Field{Type: typeRef},
				"isExported": &graphql.Field{Type: nn(graphql.Boolean)},
			}
		}),
	})

	tagValue := graphql.NewObject(graphql.ObjectConfig{
		Name: "TagValue",
		Fields: graphql.Fields{
			"key":     &graphql.Field{Type: nn(graphql.String)},
			"value":   &graphql.Field{Type: nn(graphql.String)},
			"options": &graphql.Field{Type: nnList(graphql.String)},
		},
	})

	structTag := graphql.NewObject(graphql.ObjectConfig{
		Name: "StructTag",
		Description: "Go's own metadata, and nothing like an attribute or a " +
			"decorator: a raw string on a FIELD, holding key:\"value\" pairs that " +
			"every library parses for itself. It carries no type, takes no " +
			"arguments, and cannot appear on a type or a method. `json:\"sku," +
			"omitempty\"` is the single most important fact about a field for a " +
			"generator emitting a wire shape.",
		Fields: graphql.Fields{
			"raw":   &graphql.Field{Type: nn(graphql.String), Description: "The truth."},
			"pairs": &graphql.Field{Type: nnList(tagValue), Description: "The convenience."},
		},
	})

	field := graphql.NewObject(graphql.ObjectConfig{
		Name: "Field",
		Fields: graphql.Fields{
			"name": &graphql.Field{Type: nn(graphql.String)},
			"type": &graphql.Field{Type: nn(typeRef)},
			"isExported": &graphql.Field{
				Type:        nn(graphql.Boolean),
				Description: "Spelling, not a keyword: `Name` is exported and `name` is not.",
			},
			"isEmbedded": &graphql.Field{
				Type:        nn(graphql.Boolean),
				Description: "Composition with promotion, which is not inheritance.",
			},
			"tag":         &graphql.Field{Type: structTag},
			"declaration": &graphql.Field{Type: span},
		},
	})

	param := graphql.NewObject(graphql.ObjectConfig{
		Name: "Param",
		Fields: graphql.Fields{
			"name": &graphql.Field{Type: nn(graphql.String)},
			"type": &graphql.Field{Type: nn(typeRef)},
		},
	})

	method := graphql.NewObject(graphql.ObjectConfig{
		Name: "Method",
		Fields: graphql.Fields{
			"name": &graphql.Field{Type: nn(graphql.String)},
			"pointerReceiver": &graphql.Field{
				Type: nn(graphql.Boolean),
				Description: "Decides the method set: a VALUE of the type does not " +
					"have pointer-receiver methods, so an interface check that " +
					"ignores this is wrong.",
			},
			"params":      &graphql.Field{Type: nnList(param)},
			"results":     &graphql.Field{Type: nnList(param), Description: "Go returns several."},
			"isExported":  &graphql.Field{Type: nn(graphql.Boolean)},
			"declaration": &graphql.Field{Type: span},
		},
	})

	// A GraphQL interface with three concrete types, not one object with a
	// kind field. Every declaration here is the same Go struct, so `IsTypeOf`
	// switches on the discriminator the model already carries — the extra
	// work is the point: collapsing three declaration kinds into one object
	// is exactly the lowering this schema forbids, and a caller writing
	// `... on StructDecl { fields { … } }` is asking a question the type
	// system should answer.
	declFields := func() graphql.Fields {
		return graphql.Fields{
			"name":         &graphql.Field{Type: nn(graphql.String)},
			"package":      &graphql.Field{Type: nn(graphql.String)},
			"isExported":   &graphql.Field{Type: nn(graphql.Boolean)},
			"declarations": &graphql.Field{Type: nnList(span)},
			"methods":      &graphql.Field{Type: nnList(method)},
		}
	}

	typeDecl := graphql.NewInterface(graphql.InterfaceConfig{
		Name:   "TypeDecl",
		Fields: graphql.FieldsThunk(declFields),
	})

	isKind := func(kind string) graphql.IsTypeOfFn {
		return func(p graphql.IsTypeOfParams) bool {
			d, ok := p.Value.(Decl)
			return ok && d.Typename == kind
		}
	}

	structDecl := graphql.NewObject(graphql.ObjectConfig{
		Name:       "StructDecl",
		Interfaces: []*graphql.Interface{typeDecl},
		IsTypeOf:   isKind("StructDecl"),
		Fields: graphql.FieldsThunk(func() graphql.Fields {
			f := declFields()
			f["fields"] = &graphql.Field{Type: nnList(field)}
			return f
		}),
	})

	interfaceDecl := graphql.NewObject(graphql.ObjectConfig{
		Name: "InterfaceDecl",
		Description: "Satisfied silently, like TypeScript's — but the METHOD SET " +
			"is the whole test, and it differs between a value and a pointer.",
		Interfaces: []*graphql.Interface{typeDecl},
		IsTypeOf:   isKind("InterfaceDecl"),
		Fields: graphql.FieldsThunk(func() graphql.Fields {
			f := declFields()
			f["methodNames"] = &graphql.Field{Type: nnList(graphql.String)}
			return f
		}),
	})

	namedDecl := graphql.NewObject(graphql.ObjectConfig{
		Name: "NamedDecl",
		Description: "`type Sku string` — a DEFINED type. Not an alias, not a " +
			"struct, and Go's third declaration kind: it has its own method set " +
			"and is not assignable to its underlying type without a conversion.",
		Interfaces: []*graphql.Interface{typeDecl},
		IsTypeOf:   isKind("NamedDecl"),
		Fields: graphql.FieldsThunk(func() graphql.Fields {
			f := declFields()
			f["underlying"] = &graphql.Field{Type: nn(typeRef)}
			return f
		}),
	})

	reference := graphql.NewObject(graphql.ObjectConfig{
		Name: "Reference",
		Description: "One use of a symbol, and WHO used it. A bare location " +
			"cannot answer 'is this read by the database layer'; the referring " +
			"declaration and its package can, which is the form a generator's " +
			"exceptions are actually written in.",
		Fields: graphql.Fields{
			"span":            &graphql.Field{Type: nn(span)},
			"fromPackage":     &graphql.Field{Type: nn(graphql.String)},
			"fromDeclaration": &graphql.Field{Type: nn(graphql.String)},
			"isWrite": &graphql.Field{
				Type:        nn(graphql.Boolean),
				Description: "The left side of an assignment. `nothing outside the domain may SET this` needs reads and writes told apart.",
			},
		},
	})

	implementor := graphql.NewObject(graphql.ObjectConfig{
		Name: "Implementor",
		Description: "Who satisfies an interface, and HOW. `var _ I = T{}` and " +
			"`var _ I = &T{}` are different programs, so a generator that emits " +
			"one where the other was needed is broken.",
		Fields: graphql.Fields{
			"decl":      &graphql.Field{Type: nn(typeDecl), Resolve: resolveField("decl")},
			"byValue":   &graphql.Field{Type: nn(graphql.Boolean), Resolve: resolveField("byValue")},
			"byPointer": &graphql.Field{Type: nn(graphql.Boolean), Resolve: resolveField("byPointer")},
		},
	})

	return graphql.NewSchema(graphql.SchemaConfig{
		Types: []graphql.Type{structDecl, interfaceDecl, namedDecl},
		Query: graphql.NewObject(graphql.ObjectConfig{
			Name: "Query",
			Fields: graphql.Fields{
				"version": &graphql.Field{
					Type: nn(serverInfo),
					Resolve: func(graphql.ResolveParams) (any, error) {
						return map[string]any{
							"name": "hick-model-go", "version": "0.1.0", "language": "go",
							"root": model.Root, "fileCount": model.FileCount(),
						}, nil
					},
				},
				"unresolved": &graphql.Field{
					Type:    nnList(unresolved),
					Resolve: func(graphql.ResolveParams) (any, error) { return model.Unresolved(), nil },
				},
				"types": &graphql.Field{
					Type: nnList(typeDecl),
					Args: graphql.FieldConfigArgument{
						"packageIs":    &graphql.ArgumentConfig{Type: graphql.String},
						"nameEndsWith": &graphql.ArgumentConfig{Type: graphql.String},
						"exportedOnly": &graphql.ArgumentConfig{Type: graphql.Boolean},
					},
					Resolve: func(p graphql.ResolveParams) (any, error) {
						out := []Decl{}
						for _, d := range model.Decls() {
							if v, ok := p.Args["packageIs"].(string); ok && d.Package != v {
								continue
							}
							if v, ok := p.Args["nameEndsWith"].(string); ok && !hasSuffix(d.Name, v) {
								continue
							}
							if v, ok := p.Args["exportedOnly"].(bool); ok && v && !d.IsExported {
								continue
							}
							out = append(out, d)
						}
						return out, nil
					},
				},
				"references": &graphql.Field{
					Type: nnList(reference),
					Args: graphql.FieldConfigArgument{
						"symbol": &graphql.ArgumentConfig{
							Type:        nn(graphql.String),
							Description: "`Type`, `Type.Member`, or a bare function name.",
						},
					},
					Resolve: func(p graphql.ResolveParams) (any, error) {
						return model.References(p.Args["symbol"].(string)), nil
					},
				},
				"implementors": &graphql.Field{
					Type: nnList(implementor),
					Args: graphql.FieldConfigArgument{
						"interfaceName": &graphql.ArgumentConfig{Type: nn(graphql.String)},
					},
					Resolve: func(p graphql.ResolveParams) (any, error) {
						return model.Implementors(p.Args["interfaceName"].(string)), nil
					},
				},
			},
		}),
	})
}

func hasSuffix(s, suffix string) bool {
	return len(s) >= len(suffix) && s[len(s)-len(suffix):] == suffix
}

func resolveField(name string) graphql.FieldResolveFn {
	return func(p graphql.ResolveParams) (any, error) {
		if m, ok := p.Source.(map[string]any); ok {
			return m[name], nil
		}
		if d, ok := p.Source.(Decl); ok && name == "__typename" {
			return d.Typename, nil
		}
		return nil, nil
	}
}
