<div align="center">

# Mono-class

A java bundler for CS class. Some weird class only accept single java source file. This tool automate submission process.

</div>

## How it work

1. figure out dependency by `package` keyword.
2. bundle required dependency(it would import all file even only one is used in an dictionary)
3. remove comment, visibility(interface visibility is skipped)...


## Usage

To bundle an example project:
```
src
├── CodeGenerator.java
├── Main.java
└── Syntax
    ├── Exportable.java
    ├── PropertyList.java
    ├── Token
    │   ├── ...
    └── Tree
        ├── Diagram.java
        ├── DiagramTest.java
        ├── Property
        │   ├── Argument.java
        │   ├── ...
        ├── PropertyEntry.java
        ├── ...
```

```
❯ mono-class src/CodeGenerator.java -o CodeGenerator.java
Bundling all dependency in src/CodeGenerator.java to CodeGenerator.java
Dependency solved
20147 bytes has been written to CodeGenerator.java
```

Help:

```
❯ mono-class --help
A bundler for Java

Usage: mono-class [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to source file containing desired class

Options:
  -o, --output <OUTPUT>  output location [default: Output.java]
  -v, --verbose          verbose mode
  -h, --help             Print help
  -V, --version          Print version
```