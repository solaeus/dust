# The Design of Dust

## Virtual Machine

```mermaid
---
config:
    flowchart:
        defaultRenderer: "elk"
---
flowchart TD
    A[Entry]
    subgraph JIT-Compiled
    B[Call Stack Emptiness Check]
    C[Function Dispatch]
    subgraph System Call Stack
    D[Pre-Call Checks]
    E[Direct-Call JIT Functions]
    end
    subgraph Virtual Call Stack
    F[Pre-Call Checks]
    G[Stackless JIT Function]
    end
    end
    H[Memory Management]
    I[Valid Exit]

    A --> B
    B --> C
    B --> I
    C --> D
    D --> E
    D --> H
    E --> F
    F --> G
    F --> H
    G --> B
    H --> C
```
