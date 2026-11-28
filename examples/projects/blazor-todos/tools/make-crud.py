"""Generate the todo component from the public C# code-model query."""
import argparse
from html import escape
import json
from pathlib import Path
import re
from string import Template
import subprocess

DEMO = Path(__file__).resolve().parents[1]
OUTPUT = DEMO / "app/Components/Pages/Todos.razor"
QUERY = """
{
  unresolved { message }
  type(fullName: "HickoryTodos.Models.TodoItem") {
    properties {
      name accessibility isStatic isInitOnly hasGetter hasSetter
      type { fullName }
      attributes { fullName arguments }
    }
  }
}
"""
ANNOTATION = "System.ComponentModel.DataAnnotations.EditableAttribute"
INPUTS = {"string": "InputText", "string?": "InputText", "bool": "InputCheckbox"}


def editable_properties():
    result = subprocess.run(
        ["hick", "code-model", "csharp", str(DEMO / "app/Models"), "--query", QUERY],
        cwd=DEMO, check=True, stdout=subprocess.PIPE, text=True,
    )
    result = json.loads(result.stdout)
    if result.get("errors"):
        raise ValueError(f"Code-model query failed: {result['errors']}")
    data = result["data"]
    if data["unresolved"] or not data["type"]:
        raise ValueError(f"Could not resolve the todo model: {data['unresolved']}")
    for prop in data["type"]["properties"]:
        if not any(attr["fullName"] == ANNOTATION and attr["arguments"] == ["true"]
                   for attr in prop["attributes"]):
            continue
        name = prop["name"]
        if (name in {"Id", "OwnerId"} or not re.fullmatch(r"[A-Z][A-Za-z0-9]*", name)
                or prop["accessibility"] != "PUBLIC" or prop["isStatic"] or prop["isInitOnly"]
                or not (prop["hasGetter"] and prop["hasSetter"])):
            raise ValueError(f"{name} cannot be an editable todo field.")
        field_type = prop["type"]["fullName"]
        if field_type not in INPUTS:
            raise ValueError(f"Add a control for the {field_type} property {name}.")
        label = escape(re.sub(r"(?<=[a-z0-9])(?=[A-Z])", " ", name))
        yield name, field_type, label


def generate():
    fields, headers, cells, assignments = [], [], [], []
    for name, field_type, label in editable_properties():
        control = INPUTS[field_type]
        headers.append(f'                <th scope="col">{label}</th>')
        value = f'@(item.{name} ? "Yes" : "No")' if field_type == "bool" else f"@item.{name}"
        cells.append(f"                    <td>{value}</td>")
        fields.append(
            f'        <div class="field">\n'
            f'            <label for="todo-{name}">{label}</label>\n'
            f'            <{control} id="todo-{name}" @bind-Value="draft.{name}" />\n'
            f'            <ValidationMessage For="() => draft.{name}" />\n'
            f'        </div>'
        )
        assignments.append(f"        target.{name} = source.{name};")
    if not fields:
        raise ValueError("Mark at least one todo property with Editable(true).")
    template = Template(Path(__file__).with_name("Todos.razor.template").read_text())
    return template.substitute(fields="\n".join(fields), headers="\n".join(headers),
                               cells="\n".join(cells), assignments="\n".join(assignments))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare against the constructed Razor file")
    args = parser.parse_args()
    source = generate()
    if args.check:
        if OUTPUT.read_bytes() != source.encode("utf-8"):
            parser.exit(1, "Todos.razor differs from the C# model. Refresh its literate file element.\n")
    else:
        print(source, end="")


if __name__ == "__main__":
    main()
