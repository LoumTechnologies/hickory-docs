<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/fixtures/dotnet/help-classlib.txt

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c29d5ca2aa8db6afa59b064412ad43ae1d0c0d62aadbea8e1f497f9040345b86`.

<book:entry path="crates/hickory-cli/tests/fixtures/dotnet/help-classlib.txt">
Class Library (C#)
Author: Microsoft
Description: A project for creating a class library that targets .NET or .NET Standard

Usage:
  dotnet new classlib [options] [template options]

Options:
  -n, --name <name>       The name for the output being created. If no name is 
                          specified, the name of the output directory is used.
  -o, --output <output>   Location to place the generated output.
  --dry-run               Displays a summary of what would happen if the given 
                          command line were run if it would result in a 
                          template creation. [default: False]
  --force                 Forces content to be generated even if it would 
                          change existing files. [default: False]
  --no-update-check       Disables checking for the template package updates 
                          when instantiating a template. [default: False]
  --project <project>     The project that should be used for context 
                          evaluation.
  -lang, --language <C#>  Specifies the template language to instantiate.
  --type <project>        Specifies the template type to instantiate.

Template options:
  -f, --framework <choice>     The target framework for the project.
                               Type: choice
                                 net10.0         Target net10.0
                                 netstandard2.1  Target netstandard2.1
                                 netstandard2.0  Target netstandard2.0
                               Default: net10.0
  --langVersion <langVersion>  Sets the LangVersion property in the created 
                               project file
                               Type: text
  --no-restore                 If specified, skips the automatic restore of the 
                               project on create.
                               Type: bool
                               Default: false

To see help for other template languages (F#, VB), use --language option:
   dotnet new classlib -h --language F#

</book:entry>

</book:doc>
