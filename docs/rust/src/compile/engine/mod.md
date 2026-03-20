File: rust/src/compile/engine/mod.rs
Role: Compilation Engine Module Definitions

Description:
This file simply exposes the internal compilation engine modules responsible for managing the compilation pipeline stream and verifying the integrity of physical audio tags.

Imports:
None

Logic:
`pub mod stream;`
- Exposes the module that controls the parallel execution of the compilation build process and communication with external extension runtimes.
`pub mod verify;`
- Exposes the module that checks if the physical tags embedded in the audio files match the newly compiled metadata.
