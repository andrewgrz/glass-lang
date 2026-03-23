# glass-lang
Glass lang. Clear and transparent typing compiled to WebAssembly. 

## Language Details

* Static Typing powered by Cubic Biunification
* Compiled to WebAssembly - supported in WASI and Web browser targets
* Monorepo configuration supported by default for simple frontend and backend targets
* Mutability by request - default is immutable

## Syntax

### Variables

```glass
// a is inferred to be int
let a = 5;   

// b is a float
let b = 3.14

// c is inferred to be a string
let c = "Hello, world!";
```

### Functions


```glass
/// Simple addition function
def adder(a, b) { // A and B both have a Use of the Add trait; return Value is a type that can be added
  a + b           // since no semicolon at the end, the last expression is returned
}

let result = adder(1, 2);  // result is an Int since the arguments are Ints and adding ints returns an int
```

## Repository Information
 
### Crates

#### glass-syntax

AST + lexer + parser + diagnostics

#### glass-semantics

Name resolution + type inference + monomorphization

#### glass-codegen

WebAssembly emission

#### glass-driver

Pipeline + workspace + config

#### glass-cli

Command line interface to run the compiler 
