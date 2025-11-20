# Programming Language Features Checklist for Dust

## Data Types

| Feature                | Implemented | Fully Tested |
| ---------------------- | ----------- | ------------ |
| **Boolean (`bool`)**   | ✓ Yes       | ✓ Yes        |
| **Byte (`byte`)**      | ✓ Yes       | ✓ Yes        |
| **Character (`char`)** | ✓ Yes       | ✓ Yes        |
| **Float (`float`)**    | ✓ Yes       | ✓ Yes        |
| **Integer (`int`)**    | ✓ Yes       | ✓ Yes        |
| **String (`str`)**     | ✓ Yes       | ✓ Yes        |
| **List/Array**         | ✓ Yes       | Ｘ No        |
| **Function Types**     | ✓ Yes       | ✓ Yes        |

## Literals & Constants

| Feature                 | Implemented | Fully Tested |
| ----------------------- | ----------- | ------------ |
| **Boolean literals**    | ✓ Yes       | ✓ Yes        |
| **Byte literals (hex)** | ✓ Yes       | ✓ Yes        |
| **Character literals**  | ✓ Yes       | ✓ Yes        |
| **Float literals**      | ✓ Yes       | ✓ Yes        |
| **Integer literals**    | ✓ Yes       | ✓ Yes        |
| **String literals**     | ✓ Yes       | ✓ Yes        |
| **List literals**       | ✓ Yes       | X No         |

## Variables & Declarations

| Feature                           | Implemented | Fully Tested |
| --------------------------------- | ----------- | ------------ |
| **`let` statement (immutable)**   | ✓ Yes       | ✓ Yes        |
| **`let mut` statement (mutable)** | ✓ Yes       | ✓ Yes        |
| **Variable reassignment**         | ✓ Yes       | ✓ Yes        |
| **Type inference**                | ✓ Yes       | ✓ Yes        |

## Arithmetic Operations

| Feature                  | Implemented | Fully Tested |
| ------------------------ | ----------- | ------------ |
| **Addition (`+`)**       | ✓ Yes       | ✓ Yes        |
| **Subtraction (`-`)**    | ✓ Yes       | ✓ Yes        |
| **Multiplication (`*`)** | ✓ Yes       | ✓ Yes        |
| **Division (`/`)**       | ✓ Yes       | ✓ Yes        |
| **Modulo (`%`)**         | ✓ Yes       | ✓ Yes        |
| **Exponentiation (`^`)** | ✓ Yes       | X No         |
| **Negation (unary `-`)** | ✓ Yes       | ✓ Yes        |

## Compound Assignment Operators

| Feature                              | Implemented | Fully Tested |
| ------------------------------------ | ----------- | ------------ |
| **Addition assignment (`+=`)**       | ✓ Yes       | ? Partial    |
| **Subtraction assignment (`-=`)**    | ✓ Yes       | ? Partial    |
| **Multiplication assignment (`*=`)** | ✓ Yes       | ? Partial    |
| **Division assignment (`/=`)**       | ✓ Yes       | ? Partial    |
| **Modulo assignment (`%=`)**         | ✓ Yes       | ? Partial    |

## Comparison Operations

| Feature                          | Implemented | Fully Tested |
| -------------------------------- | ----------- | ------------ |
| **Equal (`==`)**                 | ✓ Yes       | ✓ Yes        |
| **Not equal (`!=`)**             | ✓ Yes       | ✓ Yes        |
| **Greater than (`>`)**           | ✓ Yes       | ✓ Yes        |
| **Less than (`<`)**              | ✓ Yes       | ✓ Yes        |
| **Greater than or equal (`>=`)** | ✓ Yes       | ✓ Yes        |
| **Less than or equal (`<=`)**    | ✓ Yes       | ✓ Yes        |

## Logical Operations

| Feature                 | Implemented | Fully Tested |
| ----------------------- | ----------- | ------------ |
| **Logical AND (`&&`)**  | ✓ Yes       | ✓ Yes        |
| **Logical OR (`\|\|`)** | ✓ Yes       | ✓ Yes        |
| **Logical NOT (`!`)**   | ✓ Yes       | ✓ Yes        |

## Control Flow

| Feature                    | Implemented | Fully Tested |
| -------------------------- | ----------- | ------------ |
| **`if` expression**        | ✓ Yes       | ✓ Yes        |
| **`else` expression**      | ✓ Yes       | ✓ Yes        |
| **`else if` chains**       | ✓ Yes       | ✓ Yes        |
| **`while` loop**           | ✓ Yes       | ✓ Yes        |
| **`loop` (infinite loop)** | ? Partial   | X No         |
| **`break` statement**      | ✓ Yes       | ? Partial    |
| **`return` statement**     | ✓ Yes       | ✓ Yes        |

## Functions

| Feature                         | Implemented | Fully Tested |
| ------------------------------- | ----------- | ------------ |
| **Function declaration**        | ✓ Yes       | ✓ Yes        |
| **Function calls**              | ✓ Yes       | ✓ Yes        |
| **Function parameters**         | ✓ Yes       | ✓ Yes        |
| **Function return types**       | ✓ Yes       | ✓ Yes        |
| **Type parameters (generics)**  | ✓ Yes       | ? Partial    |
| **Anonymous/closure functions** | ✓ Yes       | ? Partial    |
| **Recursion**                   | ✓ Yes       | ? Partial    |
| **Native functions**            | ✓ Yes       | ? Partial    |

## Module System

| Feature                           | Implemented | Fully Tested |
| --------------------------------- | ----------- | ------------ |
| **Module declaration (`mod`)**    | ✓ Yes       | X No         |
| **Public modules (`pub mod`)**    | ✓ Yes       | X No         |
| **Use/import statements (`use`)** | ✓ Yes       | X No         |
| **Public use statements**         | ✓ Yes       | X No         |
| **Module paths**                  | ✓ Yes       | X No         |

## Other Features

| Feature                       | Implemented | Fully Tested |
| ----------------------------- | ----------- | ------------ |
| **Block expressions**         | ✓ Yes       | ✓ Yes        |
| **Grouped expressions `()`**  | ✓ Yes       | ? Partial    |
| **Type casting (`as`)**       | ✓ Yes       | ? Partial    |
| **List indexing**             | ✓ Yes       | X No         |
| **String concatenation**      | ✓ Yes       | ✓ Yes        |
| **Scope and shadowing**       | ✓ Yes       | ✓ Yes        |
| **Public visibility (`pub`)** | ✓ Yes       | X No         |
| **Main function**             | ✓ Yes       | ✓ Yes        |

## Advanced/Future Features

| Feature                     | Implemented | Fully Tested |
| --------------------------- | ----------- | ------------ |
| **Ranges (`..`)**           | ? Partial   | X No         |
| **Threading/concurrency**   | ? Partial   | X No         |
| **Standard library access** | ? Partial   | X No         |
| **Operator expressions**    | ✓ Yes       | X No         |

## Legend

- ✓ **Yes**: Feature is implemented and has comprehensive tests
- ? **Partial**: Feature is implemented but testing is incomplete or only demonstrated in examples
- X **No**: Feature may be defined but lacks tests, or is not fully implemented
