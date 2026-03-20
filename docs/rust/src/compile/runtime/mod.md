File: rust/src/compile/runtime/mod.rs
Role: Extension Runtime Definition

Description:
A simple index file defining the modules responsible for creating external environments and launching external code kernels (like Python or Node.js) that execute user extensions.

Imports:
None

Logic:
`pub mod kernel; pub mod nix;`
- Exposes the kernel bootstrapper and the Nix dependency resolver logic to the compiler.
