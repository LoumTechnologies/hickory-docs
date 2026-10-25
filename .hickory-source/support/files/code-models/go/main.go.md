<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# code-models/go/main.go

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e393982176182eec2634261dded04aaa1b2b30c637fdcd7903489dc87b616eab`.

<book:entry path="code-models/go/main.go">
// `hick-model-go <root>` — the Go code model, served as GraphQL.
//
// One JSON object per line on stdin, one per line on stdout. Diagnostics to
// stderr; stdout carries the protocol and nothing else.

package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/graphql-go/graphql"
)

func main() {
	root := "."
	if len(os.Args) > 1 {
		root = os.Args[1]
	}
	info, err := os.Stat(root)
	if err != nil || !info.IsDir() {
		fmt.Fprintf(os.Stderr, "hick-model-go: no such directory: %s\n", root)
		fmt.Fprintln(os.Stderr, "  Pass the source root to model, e.g. `hick-model-go ./internal`.")
		os.Exit(2)
	}

	model, err := Load(root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "hick-model-go: could not read %s: %v\n", root, err)
		os.Exit(2)
	}
	schema, err := buildSchema(model)
	if err != nil {
		fmt.Fprintf(os.Stderr, "hick-model-go: schema: %v\n", err)
		os.Exit(2)
	}

	fmt.Fprintf(os.Stderr, "hick-model-go: %d file(s) under %s; ready\n", model.FileCount(), model.Root)
	for i, gap := range model.Unresolved() {
		if i >= 5 {
			break
		}
		fmt.Fprintf(os.Stderr, "  unresolved: %s\n", gap.Message)
	}

	in := bufio.NewScanner(os.Stdin)
	in.Buffer(make([]byte, 0, 1<<20), 1<<24)
	out := bufio.NewWriter(os.Stdout)
	for in.Scan() {
		line := strings.TrimSpace(in.Text())
		if line == "" {
			continue
		}
		var request struct {
			Query     string         `json:"query"`
			Variables map[string]any `json:"variables"`
		}
		var body []byte
		if err := json.Unmarshal([]byte(line), &request); err != nil {
			// A malformed request must not end the session: the caller is a
			// generator being written, and being written means getting it wrong.
			body, _ = json.Marshal(map[string]any{
				"errors": []map[string]string{{"message": err.Error()}},
			})
		} else {
			result := graphql.Do(graphql.Params{
				Schema:         schema,
				RequestString:  request.Query,
				VariableValues: request.Variables,
			})
			body, _ = json.Marshal(result)
		}
		out.Write([]byte(strings.ReplaceAll(string(body), "\n", " ")))
		out.WriteByte('\n')
		out.Flush()
	}
}
</book:entry>

</book:doc>
